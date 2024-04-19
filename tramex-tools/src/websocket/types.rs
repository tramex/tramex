use crate::websocket::onelog::OneLog;

// deserialize the message
#[derive(serde::Deserialize, Debug)]
pub struct WebSocketLog {
    pub message: String,         // Same as request
    pub message_id: Option<u64>, //Any type, force as string // Same as in request.
    pub time: f64, // Number representing time in seconds since start of the process. // Usefull to send command with absolute time.
    pub utc: f64,  //Number representing UTC seconds.
    pub logs: Vec<OneLog>,
}

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    ERROR = 1,
    WARN = 2,
    INFO = 3,
    DEBUG = 4,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub enum SourceLog {
    ENB,
    MME,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub enum Direction {
    UL,
    DL,
    FROM,
    TO,
}

impl<'de> serde::Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let a = u8::deserialize(deserializer)?;
        match a {
            1 => Ok(LogLevel::ERROR),
            2 => Ok(LogLevel::WARN),
            3 => Ok(LogLevel::INFO),
            4 => Ok(LogLevel::DEBUG),
            _ => Ok(LogLevel::INFO), // default
        }
    }
}
