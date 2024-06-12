//! Error handling for Tramex Tools

#[derive(serde::Deserialize, Debug, Clone)]
/// Error codes for Tramex Tools
pub enum ErrorCode {
    /// Not set
    NotSet = 0,

    /// WebSocket: Failed to connect
    WebSocketFailedToConnect,

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

    /// Hexe decoding failed
    HexeDecodingError,

    /// File : End of the file
    EndOfFile,

    /// File: Error while parsing the file
    FileParsing,

    /// Request error
    RequestError,
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
            Self::WebSocketFailedToConnect => "WebSocket: Failed to connect",
            Self::WebSocketErrorEncodingMessage => "WebSocket: Error encoding message",
            Self::WebSocketErrorDecodingMessage => "WebSocket: Error decoding message",
            Self::WebSocketUnknownMessageReceived => "WebSocket: Unknown message received",
            Self::WebSocketUnknownBinaryMessageReceived => "WebSocket: Unknown binary message received",
            Self::WebSocketError => "WebSocket: Error",
            Self::WebSocketClosed => "WebSocket: Closed",
            Self::WebSocketErrorClosing => "WebSocket: Error closing",
            Self::FileNotSelected => "File: No file selected",
            Self::FileErrorReadingFile => "File: Error reading file",
            Self::FileNotReady => "File: Not ready",
            Self::FileInvalidEncoding => "File: Invalid encoding (wrong UTF-8)",
            Self::NotSet => "Error code not set, please create an issue",
            Self::HexeDecodingError => "Hexe decoding error",
            Self::EndOfFile => "End of File",
            Self::FileParsing => "File: Parsing error",
            Self::RequestError => "Request error",
        };
        write!(f, "{}", str)
    }
}

impl ErrorCode {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::FileInvalidEncoding | Self::WebSocketClosed)
    }
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
/// Error structure for Tramex Tools
pub struct TramexError {
    /// Error message (human readable)
    pub message: String,

    /// Error code
    code: ErrorCode,
}

impl TramexError {
    /// Create a new error
    pub fn new(message: String, code: ErrorCode) -> Self {
        log::debug!("Error: {} - {}\n{}", code, message, std::backtrace::Backtrace::capture());
        Self { message, code }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.code.is_recoverable()
    }

    /// Get the error message
    pub fn get_msg(&self) -> String {
        format!("[{}] {}", self.code, self.code)
    }

    /// Get the error code
    pub fn get_code(&self) -> ErrorCode {
        self.code.clone()
    }
}
