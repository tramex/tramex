#[derive(serde::Deserialize, Debug, Clone)]
pub enum ErrorCode {
    NotSet = 0,
    WebScoketFailedToConnect,
    WebSocketErrorEncodingMessage,
    WebSocketErrorDecodingMessage,
    WebSocketUnknownMessageReceived,
    WebSocketUnknownBinaryMessageReceived,
    WebSocketError,
    WebSocketClosed,
    WebSocketErrorClosing,
    FileNoFileSelected,
    FileErrorReadingFile,
    FileNotReady,
    FileInvalidEncoding,
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self::NotSet
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self as &dyn std::fmt::Debug)
    }
}

impl ErrorCode {
    pub fn to_string(&self) -> String {
        match self {
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
            Self::FileNoFileSelected => "File: No file selected".to_owned(),
            Self::FileErrorReadingFile => "File: Error reading file".to_owned(),
            Self::FileNotReady => "File: Not ready".to_owned(),
            Self::FileInvalidEncoding => "File: Invalid encoding (wrong UTF-8)".to_owned(),
            Self::NotSet => "Error code not set, please create an issue".to_owned(),
        }
    }
    pub fn is_recoverable(&self) -> bool {
        match self {
            _ => true,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct TramexError {
    pub message: String,
    pub code: ErrorCode,
}

impl TramexError {
    pub fn new(message: String, code: ErrorCode) -> Self {
        Self {
            message,
            code,
            ..Default::default()
        }
    }

    pub fn is_recoverable(&self) -> bool {
        self.code.is_recoverable()
    }

    pub fn get_code(&self) -> String {
        format!("[{}] {}", self.code, self.code.to_string())
    }
}
