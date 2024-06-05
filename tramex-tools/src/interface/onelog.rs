//! This module contains the definition of the OneLog struct.

use crate::errors::TramexError;
use crate::interface::functions::extract_hexe;

use crate::interface::{
    layer::Layer,
    types::{Direction, LogLevel, SourceLog},
};

#[derive(serde::Deserialize, Debug)]
/// Data structure to store the log.
pub struct OneLog {
    /// Each item is a string representing a line of log.
    pub data: Vec<String>,

    /// Milliseconds since January 1st 1970.
    pub timestamp: u64,

    /// log layer
    pub layer: Layer,

    /// Log level: error, warn, info or debug.
    pub level: LogLevel,

    ///  Log direction: UL, DL, FROM or TO.
    pub dir: Option<Direction>,

    /// cell id
    pub cell: Option<u64>,

    /// channel
    pub channel: Option<String>,

    /// Source of the log.
    pub src: SourceLog,

    /// index
    pub idx: u64,
}

impl OneLog {
    /// Extract the hexadecimal representation of the log.
    /// # Errors
    /// Returns a TramexError if the hexe representation could not be extracted.
    pub fn extract_hexe(&self) -> Result<Vec<u8>, TramexError> {
        extract_hexe(&self.data)
    }

    /// Extract the canal message of the log.
    pub fn extract_canal_msg(&self) -> Option<String> {
        // TODO implement this function correctly
        if let Some(data_line) = self.data.first() {
            log::debug!("{:?}", data_line);
            return Some(data_line.to_owned());
        }
        None
    }
}
