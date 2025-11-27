//! Parser for NAS traces
use super::ParsingError;
use crate::data::{AdditionalInfos, Trace};
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::FileParser;

#[derive(Debug, Clone)]
/// Data structure to store the NAS message type
pub struct NASInfos {
    /// Direction of the message (UL/DL)
    pub direction: Direction,

    /// Message type (e.g., "Service request", "Authentication response")
    pub message_type: String,
}

/// NAS Parser
pub struct NASParser;

impl NASParser {
    fn parse_lines(lines: &[String]) -> Result<Vec<String>, ParsingError> {
        Ok(lines.to_vec())
    }
}

impl FileParser for NASParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        let line = &lines[0];
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        // Example: "13:20:58.310 [NAS] UL 0048 5GMM: Service request"
        // Parts: ["13:20:58.310", "[NAS]", "UL", "0048", "5GMM:", "Service", "request"]
        if parts.len() < 5 {
            return Err(ParsingError::new("Could not find enough (5) parameters for NAS".to_string(), 1));
        }
        
        let direction_result = Direction::from_str(parts[2]);
        let direction = match direction_result {
            Ok(d) => d,
            Err(_) => {
                return Err(ParsingError::new(
                    format!("The direction could not be parsed in the part {:?} of {}", parts[2], line),
                    1,
                ));
            }
        };
        
        // Message type is everything after the protocol identifier (5GMM:, etc.)
        let message_type = if parts.len() > 5 {
            parts[5..].join(" ")
        } else {
            parts.get(4).unwrap_or(&"Unknown").trim_end_matches(':').to_string()
        };
        
        Ok(AdditionalInfos::NASInfos(NASInfos {
            direction,
            message_type,
        }))
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let additional_infos = match Self::parse_additional_infos(lines) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };
        
        let text = match Self::parse_lines(lines) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        
        let trace = Trace {
            timestamp: 0,
            layer: Layer::NAS,
            additional_infos,
            text: Some(text),
        };
        Ok(trace)
    }
}
