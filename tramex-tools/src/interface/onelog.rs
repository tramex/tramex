//! This module contains the definition of the OneLog struct.

use std::str::FromStr;

use crate::data::{AdditionalInfos, Trace};
use crate::errors::TramexError;
use crate::interface::functions::extract_hexe;

use crate::interface::{layer::Layer, types::SourceLog};
use crate::tramex_error;

use super::parser::parser_rrc::RRCInfos;
use super::parser::parser_nas::NASInfos;
use super::parser::parser_basic::BasicParser;
use super::types::Direction; // to use the FileParser trait and implementations

#[derive(serde::Deserialize, Debug)]
/// Data structure to store the log.
pub struct OneLog {
    /// Each item is a string representing a line of log.
    pub data: Vec<String>,

    /// Milliseconds since January 1st 1970.
    pub timestamp: i64,

    /// log layer
    pub layer: Layer,

    /// Source of the log.
    pub src: SourceLog,

    /// index
    pub idx: u64,

    /// index
    pub dir: Option<String>,
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
            log::debug!("{data_line:?}");
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
                // log::debug!("self: {:?}", self);
                let dir = match &self.dir {
                    Some(opt_dir) => match Direction::from_str(opt_dir) {
                        Ok(d) => d,
                        Err(_) => {
                            log::debug!("Direction: {:?}", self.dir);
                            return Err(tramex_error!(
                                format!("Can't format direction {}", opt_dir),
                                crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                            ));
                        }
                    },
                    None => {
                        return Err(tramex_error!(
                            "Direction not found".to_owned(),
                            crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                        ));
                    }
                };
                let firs_line = self.data[0].split(':').collect::<Vec<&str>>();
                if firs_line.len() < 2 {
                    return Err(tramex_error!(
                        format!("Invalid first line {}", self.data[0]),
                        crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                    ));
                }
                let rrc: RRCInfos = RRCInfos {
                    direction: dir,
                    canal: firs_line[0].to_owned(),
                    canal_msg: firs_line[1][1..].to_owned(),
                };
                let infos = AdditionalInfos::RRCInfos(rrc);
                let trace = Trace {
                    timestamp: self.timestamp,
                    layer: Layer::RRC,
                    additional_infos: infos,
                    text: Some(self.data[1..].iter().map(|x| x.to_string()).collect()),
                };
                Ok(trace)
            }
            Layer::NAS => {
                let dir = match &self.dir {
                    Some(opt_dir) => match Direction::from_str(opt_dir) {
                        Ok(d) => d,
                        Err(_) => {
                            return Err(tramex_error!(
                                format!("Can't format direction {}", opt_dir),
                                crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                            ));
                        }
                    },
                    None => {
                        return Err(tramex_error!(
                            "Direction not found".to_owned(),
                            crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                        ));
                    }
                };
                
                // First line contains the message type
                // Example: "5GMM: Service request" or just "Service request"
                let message_type = if self.data.is_empty() {
                    "Unknown".to_string()
                } else {
                    // Remove protocol prefix if present (e.g., "5GMM: ")
                    let first_line = &self.data[0];
                    if let Some(colon_pos) = first_line.find(':') {
                        first_line[colon_pos + 1..].trim().to_string()
                    } else {
                        first_line.trim().to_string()
                    }
                };
                
                let nas = NASInfos {
                    direction: dir,
                    message_type,
                };
                let infos = AdditionalInfos::NASInfos(nas);
                let trace = Trace {
                    timestamp: self.timestamp,
                    layer: Layer::NAS,
                    additional_infos: infos,
                    text: Some(self.data[1..].iter().map(|x| x.to_string()).collect()),
                };
                Ok(trace)
            }
            _ => {
                // Use BasicParser for all other layers (PHY, RLC, MAC, PDCP, SDAP, etc.)
                let mut trace = BasicParser::parse_with_layer(&self.data, self.layer.clone())
                    .map_err(|e| tramex_error!(
                        e.message,
                        crate::errors::ErrorCode::ParsingLayerNotImplemented
                    ))?;
                // Set the timestamp from the WebSocket log
                trace.timestamp = self.timestamp;
                Ok(trace)
            }
        }
    }
}
