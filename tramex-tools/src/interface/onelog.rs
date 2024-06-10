//! This module contains the definition of the OneLog struct.

use crate::data::Trace;
use crate::errors::TramexError;
use crate::interface::functions::extract_hexe;

use crate::interface::{layer::Layer, types::SourceLog};

use super::parser::parser_rrc::RRCParser;
use super::parser::{parsing_error_to_tramex_error, FileParser}; // to use the FileParser trait and implementations

#[derive(serde::Deserialize, Debug)]
/// Data structure to store the log.
pub struct OneLog {
    /// Each item is a string representing a line of log.
    pub data: Vec<String>,

    /// Milliseconds since January 1st 1970.
    pub timestamp: u64,

    /// log layer
    pub layer: Layer,

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

    /// Extract the data of the log.
    /// # Errors
    /// Returns a TramexError if the data could not be extracted.
    pub fn extract_data(&self) -> Result<Trace, TramexError> {
        match self.layer {
            Layer::RRC => {
                let infos = match RRCParser::parse_additional_infos(&self.data) {
                    Ok(i) => i,
                    Err(err) => {
                        return Err(parsing_error_to_tramex_error(err, 0));
                    }
                };
                let trace = Trace {
                    timestamp: self.timestamp.to_owned(),
                    layer: Layer::RRC,
                    additional_infos: infos,
                    hexa: self.extract_hexe().unwrap_or_default(),
                    text: Some(self.data.iter().map(|x| x.to_string()).collect()), // maybe filter files
                };
                Ok(trace)
            }
            _ => Err(TramexError::new(
                "Layer not implemented".to_owned(),
                crate::errors::ErrorCode::WebSocketErrorDecodingMessage,
            )),
        }
    }
}
