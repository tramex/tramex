//! This module contains the types used in the websocket module.

use std::str::FromStr;

use crate::interface::onelog::OneLog;

// deserialize the message
#[derive(serde::Deserialize, Debug)]
/// LogGet struct
pub struct WebSocketLog {
    /// Same as request
    pub message: String,

    ///Any type, force as string // Same as in request.
    pub message_id: Option<u64>,

    /// Number representing time in seconds since start of the process. // Useful to send command with absolute time.
    pub time: f64,

    ///Number representing UTC seconds.
    pub utc: f64,

    /// Logs vectors
    pub logs: Vec<OneLog>,
}

/// LogGet struct
#[derive(serde::Deserialize, Debug)]
pub struct BaseMessage {
    /// Message
    pub message: String,

    /// Message ID
    pub name: String,

    /// Time
    pub time: f64,

    /// UTC
    pub utc: f64,

    /// Version
    pub version: String,
}

#[derive(Debug, PartialEq)]
/// LogLevel struct
pub enum LogLevel {
    /// Error log level
    ERROR = 1,

    /// Warning log level
    WARN = 2,

    /// Info log level
    INFO = 3,

    /// Debug log level
    DEBUG = 4,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
/// SourceLog enum
pub enum SourceLog {
    /// ENB source
    ENB,

    /// MME source
    MME,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone)]
/// Direction enum
pub enum Direction {
    #[default]
    /// Uplink direction
    UL,

    /// Downlink direction
    DL,

    /// From direction
    FROM,

    /// To direction
    TO,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(input_string: &str) -> Result<Self, Self::Err> {
        match input_string {
            "UL" => Ok(Direction::UL),
            "DL" => Ok(Direction::DL),
            "FROM" => Ok(Direction::FROM),
            "TO" => Ok(Direction::TO),
            _ => Err(()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialized_int = u8::deserialize(deserializer)?;
        match deserialized_int {
            1 => Ok(LogLevel::ERROR),
            2 => Ok(LogLevel::WARN),
            3 => Ok(LogLevel::INFO),
            4 => Ok(LogLevel::DEBUG),
            _ => Ok(LogLevel::INFO), // default
        }
    }
}
