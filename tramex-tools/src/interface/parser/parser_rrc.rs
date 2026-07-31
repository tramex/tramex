//! Parser for RRC traces
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::data::{AdditionalInfos, Trace};
use crate::interface::association::TraceRelation;
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::{FileParser, LayerParser, ParsedHeader};

#[derive(Debug, Clone)]
/// Data structure to store the message type (from the amarisoft API)
pub struct RRCInfos {
    /// Direction of the message.
    pub direction: Direction,

    /// canal of the message.
    pub canal: String,

    /// Message of the canal.
    pub canal_msg: String,
}

/// RRC Parser
pub struct RRCParser;

impl RRCParser {
    /// Parse the lines
    fn parse_lines(lines: &[String]) -> Vec<String> {
        lines.to_vec()
    }
}

impl FileParser for RRCParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        let line = &lines[0];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(ParsingError::new("Could not find enough (5) parameters".to_string(), 1));
        }
        let direction_result = Direction::from_str(parts[2]);
        let binding: String = parts[5..].join(" ");
        let concatenated: Vec<&str> = binding.split(':').collect();
        let direction = match direction_result {
            Ok(d) => d,
            Err(_) => {
                return Err(ParsingError::new(
                    format!("The direction could not be parsed in the part {:?} of {}", parts[2], line),
                    1,
                ));
            }
        };
        if concatenated.len() < 2 || concatenated[0].is_empty() || concatenated[1].is_empty() {
            return Err(ParsingError::new(
                "The canal and/or canal message could not be parsed".to_string(),
                1,
            ));
        }
        Ok(AdditionalInfos::RRCInfos(RRCInfos {
            direction,
            canal: concatenated[0].to_owned(),
            canal_msg: concatenated[1].trim_start().to_owned(),
        }))
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let mtype = match Self::parse_additional_infos(lines) {
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
            layer: Layer::RRC,
            additional_infos: mtype,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}

impl LayerParser for RRCParser {
    /// Parse RRC payload lines.
    /// data_lines[0] should be "CANAL: message" (e.g. "DCCH-NR: RRC release")
    fn parse_layer(header: &ParsedHeader, data_lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        if data_lines.is_empty() {
            return Err(ParsingError::new("RRC: empty data lines".to_string(), 0));
        }
        let first_line = &data_lines[0];
        let concatenated: Vec<&str> = first_line.split(':').collect();
        if concatenated.len() < 2 || concatenated[0].is_empty() || concatenated[1].is_empty() {
            return Err(ParsingError::new(
                format!("RRC: cannot parse canal:message from '{}'", first_line),
                0,
            ));
        }
        Ok(AdditionalInfos::RRCInfos(RRCInfos {
            direction: header.direction.clone(),
            canal: concatenated[0].to_owned(),
            canal_msg: concatenated[1].trim_start().to_owned(),
        }))
    }
}

/// Counting Brackets
#[inline]
pub fn count_brackets(hay: &str) -> i16 {
    let mut count: i16 = 0;
    for ch in hay.chars() {
        match ch {
            '{' => count += 1,
            '}' => count -= 1,
            _ => (),
        }
    }
    count
}
