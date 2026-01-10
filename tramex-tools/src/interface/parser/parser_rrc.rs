//! Parser for RRC traces
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::interface::association::TraceRelation;
use crate::data::{AdditionalInfos, Trace};
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::FileParser;

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
    fn parse_lines(lines: &[String]) -> Result<Vec<String>, ParsingError> {
        Ok(lines.to_vec())
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
        let text = match Self::parse_lines(lines) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        let binary = extract_binary_from_lines(lines);
        let trace = Trace {
            timestamp: 0,
            layer: Layer::RRC,
            additional_infos: mtype,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
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
