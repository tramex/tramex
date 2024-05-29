use crate::functions::extract_hexe;

use crate::websocket::{
    layer::Layer,
    types::{Direction, LogLevel, SourceLog},
};

#[derive(serde::Deserialize, Debug)]
pub struct OneLog {
    pub data: Vec<String>,       // Each item is a string representing a line of log.
    pub timestamp: u64,          // Milliseconds since January 1st 1970.
    pub layer: Layer,            // log layer
    pub level: LogLevel,         // Log level: error, warn, info or debug.
    pub dir: Option<Direction>,  //  Log direction: UL, DL, FROM or TO.
    pub cell: Option<u64>,       // cell id
    pub channel: Option<String>, // channel
    pub src: SourceLog,
    pub idx: u64,
}

impl OneLog {
    pub fn extract_hexe(&self) -> Vec<u8> {
        return extract_hexe(&self.data);
    }

    pub fn extract_canal_msg(&self) -> Option<String> {
        // TODO implement this function correctly
        if let Some(data_line) = self.data.first() {
            log::debug!("{:?}", data_line);
            return Some(data_line.to_owned());
        }
        None
    }
}
