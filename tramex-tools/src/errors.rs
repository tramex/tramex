#[derive(serde::Deserialize, Debug, Default)]
pub struct TramexError {
    pub message: String,
    pub code: u32,
}

impl TramexError {
    pub fn new(message: String, code: u32) -> Self {
        Self {
            message,
            code,
            ..Default::default()
        }
    }

    pub fn is_recoverable(&self) -> bool {
        match self.code {
            0..=10 => true,
            _ => false,
        }
    }
    pub fn get_code(&self) -> String {
        match self.code {
            1 => format!("[{}] WebSocket: Failed to connect", self.code),
            2 => format!("[{}] WebSocket: Error decoding message", self.code),
            3 => format!("[{}] WebScoket: Unknown message received", self.code),
            4 => format!("[{}] WebScoket: Unknown binary message received", self.code),
            5 => format!("[{}] WebSocket: Error", self.code),
            _ => format!("[{}] Unknow code, please report an issue", self.code),
        }
    }
}
