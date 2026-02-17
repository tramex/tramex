//! Parser for NGAP traces
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::interface::association::TraceRelation;
use crate::data::{AdditionalInfos, Trace};
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::FileParser;

#[derive(Debug, Clone)]
/// Data structure to store the NGAP message type
pub struct NGAPInfos {
    /// Direction of the message (TO/FROM)
    /// Check direction NGAP: Currently TO = BST -> CN, FROM = CN -> BST
    pub direction: Direction,

    /// Message type (e.g., "UE context release complete", "Initial context setup request")
    pub message_type: String,
    
    /// Connection info (e.g., "127.0.1.100:38412")
    pub connection_info: Option<String>,
}

/// NGAP Parser
pub struct NGAPParser;

impl NGAPParser {
    fn parse_lines(lines: &[String]) -> Result<Vec<String>, ParsingError> {
        Ok(lines.to_vec())
    }
}

impl FileParser for NGAPParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        let line = &lines[0];
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        // Example: "13:20:58.310 [NGAP] TO 0080 003d 127.0.1.100:38412 UE context release complete"
        // Parts: ["13:20:58.310", "[NGAP]", "TO", "0080", "003d", "127.0.1.100:38412", "UE", "context", "release", "complete"]
        if parts.len() < 6 {
            return Err(ParsingError::new("Could not find enough (6) parameters for NGAP".to_string(), 1));
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
        
        // Connection info is typically at parts[5] (IP:port)
        let connection_info = if parts.len() > 5 && parts[5].contains(':') {
            Some(parts[5].to_string())
        } else {
            None
        };
        
        // Message type is everything after connection info
        let message_start_idx = if connection_info.is_some() { 6 } else { 5 };
        let message_type = if parts.len() > message_start_idx {
            parts[message_start_idx..].join(" ")
        } else {
            "Unknown".to_string()
        };
        
        Ok(AdditionalInfos::NGAPInfos(NGAPInfos {
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
        
        let text = lines.to_vec();
        let binary = extract_binary_from_lines(lines);
        
        // Parse timestamp from first line
        let timestamp = if let Some(first_line) = lines.first() {
            super::parse_timestamp(first_line)?
        } else {
            0
        };
        
        let trace = Trace {
            timestamp,
            layer: Layer::NGAP,
            additional_infos,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}
