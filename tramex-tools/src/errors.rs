//! Error handling for Tramex Tools

#[derive(serde::Deserialize, Debug, Clone)]
/// Error codes for Tramex Tools
pub enum ErrorCode {
    /// Not set
    NotSet = 0,

    /// WebSocket: Failed to connect
    WebScoketFailedToConnect,

    /// WebSocket: Error encoding message
    WebSocketErrorEncodingMessage,

    /// WebSocket: Error decoding message
    WebSocketErrorDecodingMessage,

    /// WebSocket: Unknown message received
    WebSocketUnknownMessageReceived,

    /// WebSocket: Unknown binary message received
    WebSocketUnknownBinaryMessageReceived,

    /// WebSocket: Error
    WebSocketError,

    /// WebSocket: Closed
    WebSocketClosed,

    /// WebSocket: Error closing
    WebSocketErrorClosing,

    /// File: No file selected
    FileNotSelected,

    /// File: Error reading file
    FileErrorReadingFile,

    /// File: Not ready
    FileNotReady,

    /// File: Invalid encoding (wrong UTF-8)
    FileInvalidEncoding,
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::NotSet
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // use to_string() to get the string representation of the error code
        let str = match self {
            Self::WebScoketFailedToConnect => "WebSocket: Failed to connect".to_owned(),
            Self::WebSocketErrorEncodingMessage => "WebSocket: Error encoding message".to_owned(),
            Self::WebSocketErrorDecodingMessage => "WebSocket: Error decoding message".to_owned(),
            Self::WebSocketUnknownMessageReceived => {
                "WebSocket: Unknown message received".to_owned()
            }
            Self::WebSocketUnknownBinaryMessageReceived => {
                "WebSocket: Unknown binary message received".to_owned()
            }
            Self::WebSocketError => "WebSocket: Error".to_owned(),
            Self::WebSocketClosed => "WebSocket: Closed".to_owned(),
            Self::WebSocketErrorClosing => "WebSocket: Error closing".to_owned(),
            Self::FileNotSelected => "File: No file selected".to_owned(),
            Self::FileErrorReadingFile => "File: Error reading file".to_owned(),
            Self::FileNotReady => "File: Not ready".to_owned(),
            Self::FileInvalidEncoding => "File: Invalid encoding (wrong UTF-8)".to_owned(),
            Self::NotSet => "Error code not set, please create an issue".to_owned(),
        };
        write!(f, "{}", str)
    }
}

impl ErrorCode {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        // match self {
        //     _ => true,
        // }
        true
    }
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
/// Error structure for Tramex Tools
pub struct TramexError {
    /// Error message (human readable)
    pub message: String,

    /// Error code
    pub code: ErrorCode,
}

impl TramexError {
    /// Create a new error
    pub fn new(message: String, code: ErrorCode) -> Self {
        Self { message, code }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.code.is_recoverable()
    }

    /// Get the error message
    pub fn get_code(&self) -> String {
        format!("[{}] {}", self.code, self.code)
    }
}
