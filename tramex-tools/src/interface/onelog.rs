//! This module contains the definition of the OneLog struct.

use std::str::FromStr;

use crate::data::{AdditionalInfos, Trace};
use crate::errors::TramexError;
use crate::interface::functions::extract_hexe;

use crate::interface::{layer::Layer, types::SourceLog};
use crate::tramex_error;

use super::parser::parser_rrc::RRCInfos;
use super::types::Direction; // to use the FileParser trait and implementations

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
                    timestamp: self.timestamp.to_owned(),
                    layer: Layer::RRC,
                    additional_infos: infos,
                    hexa: self.extract_hexe().unwrap_or_default(),
                    text: Some(self.data[1..].iter().map(|x| x.to_string()).collect()),
                };
                Ok(trace)
            }
            _ => Err(tramex_error!(
                format!("Layer {:?} not implemented", self.layer),
                crate::errors::ErrorCode::ParsingLayerNotImplemented
            )),
        }
    }
}
