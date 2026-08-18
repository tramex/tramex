//! Anthropic (Claude) connector implementation

use crate::ai::{AIConnector, AIRequest};
use crate::data::{AdditionalInfos, Trace};
use crate::errors::{ErrorCode, TramexError};

/// Anthropic API version header value
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// System prompt for Anthropic explaining Amarisoft traces
const SYSTEM_PROMPT: &str = r#"You are a telecom protocol expert specializing in 4G LTE and 5G NR analysis. You are helping a user understand traces captured from an Amarisoft base station (eNB/gNB).

When explaining a trace, structure your response in three sections:
1. **Message Overview**: What this message is, which protocol layer it belongs to, and its role in the signaling flow.
2. **Key Fields**: Explain the important fields and parameters present in the message. Use precise telecom terminology but provide brief clarifications for non-obvious terms.
3. **Protocol Context**: Where this message fits in the typical protocol procedure (e.g. attach, handover, bearer setup). Mention what typically precedes and follows it.

Be concise but technically accurate. Target approximately 200 words. Use markdown formatting for readability."#;

/// Anthropic (Claude) connector
pub struct AnthropicConnector {
    /// API endpoint
    endpoint: String,
    /// Model to use
    model: String,
}

impl AnthropicConnector {
    /// Create a new AnthropicConnector with default settings
    pub fn new() -> Self {
        Self {
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            model: "claude-3-5-haiku-latest".to_string(),
        }
    }

    /// Create an AnthropicConnector with a specific model
    pub fn with_model(model: &str) -> Self {
        Self {
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
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

impl Default for AnthropicConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl AIConnector for AnthropicConnector {
    fn name(&self) -> &'static str {
        "Anthropic"
    }

    fn build_request(&self, trace: &Trace, api_key: &str) -> Result<AIRequest, TramexError> {
        if api_key.is_empty() {
            return Err(TramexError::new(
                "API key is empty. Set it in Settings > AI.".to_string(),
                ErrorCode::RequestError,
            ));
        }

        let user_message = Self::build_user_message(trace);

        // Anthropic Messages API: system prompt is a top-level field,
        // not a message with role "system".
        let body = serde_json::json!({
            "model": self.model,
            "system": SYSTEM_PROMPT,
            "messages": [
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1024
        });

        let body_str = serde_json::to_string(&body)
            .map_err(|e| TramexError::new(format!("Failed to serialize request body: {e}"), ErrorCode::RequestError))?;

        Ok(AIRequest {
            url: self.endpoint.clone(),
            headers: vec![
                ("x-api-key".to_string(), api_key.to_string()),
                ("anthropic-version".to_string(), ANTHROPIC_VERSION.to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: body_str,
        })
    }

    fn parse_response(&self, response_body: &str) -> Result<String, TramexError> {
        let json: serde_json::Value = serde_json::from_str(response_body)
            .map_err(|e| TramexError::new(format!("Failed to parse AI response: {e}"), ErrorCode::RequestError))?;

        // Anthropic error format: {"type":"error","error":{"type":"...","message":"..."}}
        if let Some(error) = json.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .or_else(|| error.as_str())
                .unwrap_or("Unknown API error");
            return Err(TramexError::new(format!("Anthropic API error: {msg}"), ErrorCode::RequestError));
        }

        // Some error responses expose details at the top level
        if let Some(msg) = json
            .get("message")
            .and_then(|m| m.as_str())
            .or_else(|| json.get("detail").and_then(|d| d.as_str()))
        {
            return Err(TramexError::new(format!("Anthropic API error: {msg}"), ErrorCode::RequestError));
        }

        // Anthropic response format: {"content":[{"type":"text","text":"..."}], ...}
        json.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.iter().find(|block| block.get("type").and_then(|t| t.as_str()) == Some("text")))
            .and_then(|block| block.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                TramexError::new(
                    "Unexpected response format from Anthropic API".to_string(),
                    ErrorCode::RequestError,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_successful_response() {
        let connector = AnthropicConnector::new();
        let body = r#"{"content":[{"type":"text","text":"Hello"}]}"#;
        assert_eq!(connector.parse_response(body).unwrap(), "Hello");
    }

    #[test]
    fn parse_error_with_nested_message() {
        let connector = AnthropicConnector::new();
        let body = r#"{"type":"error","error":{"type":"authentication_error","message":"Invalid API key"}}"#;
        let err = connector.parse_response(body).unwrap_err();
        assert!(err.get_msg().contains("Invalid API key"));
    }

    #[test]
    fn parse_error_with_top_level_message() {
        let connector = AnthropicConnector::new();
        let body = r#"{"message":"Unauthorized","request_id":"abc"}"#;
        let err = connector.parse_response(body).unwrap_err();
        assert!(err.get_msg().contains("Unauthorized"));
    }
}
