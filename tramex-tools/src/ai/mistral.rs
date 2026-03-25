//! Mistral AI connector implementation

use crate::ai::{AIConnector, AIRequest};
use crate::data::{Trace, AdditionalInfos};
use crate::errors::{TramexError, ErrorCode};

/// System prompt for Mistral AI explaining Amarisoft traces
const SYSTEM_PROMPT: &str = r#"You are a telecom protocol expert specializing in 4G LTE and 5G NR analysis. You are helping a user understand traces captured from an Amarisoft base station (eNB/gNB).

When explaining a trace, structure your response in three sections:
1. **Message Overview**: What this message is, which protocol layer it belongs to, and its role in the signaling flow.
2. **Key Fields**: Explain the important fields and parameters present in the message. Use precise telecom terminology but provide brief clarifications for non-obvious terms.
3. **Protocol Context**: Where this message fits in the typical protocol procedure (e.g. attach, handover, bearer setup). Mention what typically precedes and follows it.

Be concise but technically accurate. Target approximately 200 words. Use markdown formatting for readability."#;

/// Mistral AI connector
pub struct MistralConnector {
    /// API endpoint
    endpoint: String,
    /// Model to use
    model: String,
}

impl MistralConnector {
    /// Create a new MistralConnector with default settings
    pub fn new() -> Self {
        Self {
            endpoint: "https://api.mistral.ai/v1/chat/completions".to_string(),
            model: "mistral-medium-latest".to_string(),
        }
    }

    /// Create a MistralConnector with a specific model
    pub fn with_model(model: &str) -> Self {
        Self {
            endpoint: "https://api.mistral.ai/v1/chat/completions".to_string(),
            model: model.to_string(),
        }
    }

    /// Build the user message from a Trace
    fn build_user_message(trace: &Trace) -> String {
        let mut parts = Vec::new();

        // Structured metadata
        parts.push(format!("Layer: {:?}", trace.layer));
        parts.push(format!("Timestamp: {}", trace.timestamp));

        // Direction and message type from additional infos
        match &trace.additional_infos {
            AdditionalInfos::RRCInfos(info) => {
                parts.push(format!("Direction: {:?}", info.direction));
                parts.push(format!("Channel/Message: {}", info.canal_msg));
            }
            AdditionalInfos::NASInfos(info) => {
                parts.push(format!("Direction: {:?}", info.direction));
                parts.push(format!("Message Type: {}", info.message_type));
            }
            AdditionalInfos::NGAPInfos(info) => {
                parts.push(format!("Direction: {:?}", info.direction));
                parts.push(format!("Message Type: {}", info.message_type));
            }
            AdditionalInfos::GTPUInfos(info) => {
                parts.push(format!("Direction: {:?}", info.direction));
                parts.push(format!("Message Type: {}", info.message_type));
            }
            AdditionalInfos::PHYInfos(info) => {
                parts.push(format!("Direction: {:?}", info.direction));
                parts.push(format!("Channel Type: {:?}", info.channel_type));
            }
            AdditionalInfos::None => {}
        }

        // Raw text
        if let Some(text_lines) = &trace.text {
            parts.push(String::new());
            parts.push("Raw trace content:".to_string());
            parts.push("```".to_string());
            for line in text_lines {
                parts.push(line.clone());
            }
            parts.push("```".to_string());
        }

        parts.join("\n")
    }
}

impl Default for MistralConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl AIConnector for MistralConnector {
    fn name(&self) -> &'static str {
        "Mistral"
    }

    fn build_request(&self, trace: &Trace, api_key: &str) -> Result<AIRequest, TramexError> {
        if api_key.is_empty() {
            return Err(TramexError::new(
                "API key is empty. Set it in Settings > AI.".to_string(),
                ErrorCode::RequestError,
            ));
        }

        let user_message = Self::build_user_message(trace);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1024
        });

        let body_str = serde_json::to_string(&body).map_err(|e| {
            TramexError::new(
                format!("Failed to serialize request body: {e}"),
                ErrorCode::RequestError,
            )
        })?;

        Ok(AIRequest {
            url: self.endpoint.clone(),
            headers: vec![
                ("Authorization".to_string(), format!("Bearer {api_key}")),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: body_str,
        })
    }

    fn parse_response(&self, response_body: &str) -> Result<String, TramexError> {
        let json: serde_json::Value = serde_json::from_str(response_body).map_err(|e| {
            TramexError::new(
                format!("Failed to parse AI response: {e}"),
                ErrorCode::RequestError,
            )
        })?;

        // Check for API error
        if let Some(error) = json.get("error") {
            let msg = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown API error");
            return Err(TramexError::new(
                format!("Mistral API error: {msg}"),
                ErrorCode::RequestError,
            ));
        }

        // Extract the assistant's message content
        json.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                TramexError::new(
                    "Unexpected response format from Mistral API".to_string(),
                    ErrorCode::RequestError,
                )
            })
    }
}
