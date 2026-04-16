//! Parser for GTPU traces
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::data::{AdditionalInfos, Trace};
use crate::interface::association::TraceRelation;
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::FileParser;

#[derive(Debug, Clone)]
/// Data structure to store the GTPU message type
pub struct GTPUInfos {
    /// Direction of the message (TO/FROM)
    pub direction: Direction,

    /// Message type (e.g., "G-PDU", "Echo Request")
    pub message_type: String,

    /// Connection info (e.g., "127.0.1.100:2152")
    pub connection_info: Option<String>,
}

/// GTPU Parser
pub struct GTPUParser;

impl GTPUParser {
    /// Function that parses the remaining lines of a log
    fn parse_lines(lines: &[String]) -> Vec<String> {
        lines.to_vec()
    }
}

impl FileParser for GTPUParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        let line = &lines[0];
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Example: "13:20:45.574 [GTPU] TO 127.0.1.100:2152 G-PDU TEID=0x5315caa3 QFI=1 SDU_len=1452: IP/TCP ..."
        // Parts: ["13:20:45.574", "[GTPU]", "TO", "127.0.1.100:2152", "G-PDU", ...]
        if parts.len() < 5 {
            return Err(ParsingError::new(
                "Could not find enough (5) parameters for GTPU".to_string(),
                1,
            ));
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

        // Connection info is typically at parts[3] (IP:port)
        let connection_info = if parts.len() > 3 && parts[3].contains(':') {
            Some(parts[3].to_string())
        } else {
            None
        };

        // Message type is at parts[4] (e.g., "G-PDU")
        let message_type = parts.get(4).unwrap_or(&"Unknown").to_string();

        Ok(AdditionalInfos::GTPUInfos(GTPUInfos {
            direction,
            message_type,
            connection_info,
        }))
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let additional_infos = match Self::parse_additional_infos(lines) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };

        let text = Self::parse_lines(lines);

        let binary = extract_binary_from_lines(lines);

        // Parse timestamp from first line
        let timestamp = if let Some(first_line) = lines.first() {
            super::parse_timestamp(first_line)?
        } else {
            0
        };

        let trace = Trace {
            timestamp,
            layer: Layer::GTPU,
            additional_infos,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}
