//! AI connector module for trace explanation
//! 
//! Provides a trait-based abstraction for AI chatbot APIs.
//! Enable with the `ai` feature flag.

pub mod mistral;

use crate::data::Trace;
use crate::errors::TramexError;

/// Represents an HTTP request to be sent to an AI API
#[derive(Debug, Clone)]
pub struct AIRequest {
    /// The full URL endpoint
    pub url: String,
    /// HTTP headers as (key, value) pairs
    pub headers: Vec<(String, String)>,
    /// JSON body as a string
    pub body: String,
}

/// Status of an AI explanation request
#[derive(Debug, Clone)]
pub enum AIExplainStatus {
    /// No request has been made
    Idle,
    /// Request is in flight
    Loading,
    /// Response received successfully
    Done(String),
    /// Request failed
    Error(String),
}

impl Default for AIExplainStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Trait abstracting an AI chatbot connector.
/// 
/// Implementations build the HTTP request and parse the response.
/// The actual HTTP call is handled by the UI layer (using ehttp).
pub trait AIConnector: Send + Sync {
    /// Human-readable name of the AI provider
    fn name(&self) -> &'static str;

    /// Build the HTTP request for explaining a trace.
    /// 
    /// # Arguments
    /// * `trace` - The trace to explain
    /// * `api_key` - The API key for authentication
    /// 
    /// # Errors
    /// Returns an error if the request cannot be built
    fn build_request(&self, trace: &Trace, api_key: &str) -> Result<AIRequest, TramexError>;

    /// Parse the API response body into a human-readable explanation.
    /// 
    /// # Arguments
    /// * `response_body` - The raw JSON response from the API
    /// 
    /// # Errors
    /// Returns an error if the response cannot be parsed
    fn parse_response(&self, response_body: &str) -> Result<String, TramexError>;
}

/// Available AI provider types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AIProvider {
    /// Mistral AI
    Mistral,
}

impl Default for AIProvider {
    fn default() -> Self {
        Self::Mistral
    }
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mistral => write!(f, "Mistral"),
        }
    }
}

/// Create an AIConnector from a provider type
pub fn create_connector(provider: &AIProvider) -> Box<dyn AIConnector> {
    match provider {
        AIProvider::Mistral => Box::new(mistral::MistralConnector::new()),
    }
}
