//! Parser for RRC traces
use super::ParsingError;
use crate::data::{AdditionalInfos, Trace};
use std::str::FromStr;

use crate::interface::{functions::extract_hexe, layer::Layer, types::Direction};

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
    /// Function that parses the hexadecimal part of a log
    fn parse_lines(lines: &[String]) -> Result<(Vec<u8>, Vec<String>), ParsingError> {
        let lines_len = lines.len();
        let mut ix = 0;
        let mut hex_str: Vec<&str> = vec![];
        while ix < lines_len {
            match lines[ix].trim_start().chars().next() {
                Some(c) => {
                    if c == '{' {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
            hex_str.push(&lines[ix]);
            ix += 1;
        }
        if ix >= lines_len {
            return Err(ParsingError::new(
                "Could not find the end of the hexadecimal".to_string(),
                ix as u64,
            ));
        }
        let hex = match extract_hexe(&hex_str) {
            Ok(h) => h,
            Err(e) => return Err(ParsingError::new(e.message, ix as u64)),
        };

        let mut end = false;
        let mut brackets: i16 = 0;
        let start_block = ix;
        while (ix < lines_len) && !end {
            brackets += count_brackets(&lines[ix]);
            ix += 1;
            if brackets == 0 {
                end = true;
            }
        }
        if ix >= lines_len && !end {
            return Err(ParsingError::new(
                "Could not parse the JSON like part, missing closing }".to_string(),
                ix as u64,
            ));
        }
        let text = lines[start_block..ix].iter().map(|s| s.to_string()).collect();
        Ok((hex, text))
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
                ))
            }
        };
        if concatenated.len() < 2 || concatenated[0].is_empty() || concatenated[1].is_empty() {
            return Err(ParsingError::new(
                "The canal and/or canal message could not be parsed".to_string(),
                1,
            ));
        }
        return Ok(AdditionalInfos::RRCInfos(RRCInfos {
            direction,
            canal: concatenated[0].to_owned(),
            canal_msg: concatenated[1].trim_start().to_owned(),
        }));
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let mtype = match Self::parse_additional_infos(&lines) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };
        let (hexa, text) = match Self::parse_lines(&lines[1..]) {
            Ok((h, t)) => (h, t),
            Err(e) => {
                return Err(e);
            }
        };
        let trace = Trace {
            timestamp: 0,
            layer: Layer::RRC,
            additional_infos: mtype,
            hexa,
            text: Some(text),
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
