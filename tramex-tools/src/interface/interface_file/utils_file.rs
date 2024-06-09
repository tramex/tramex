//! utils functions for file interface

use std::str::FromStr;

use crate::errors::ErrorCode;
use crate::interface::parser::{parsing_error_to_tramex_error, FileParser}; // to use the FileParser trait and implementations
use crate::{
    data::Trace,
    errors::TramexError,
    interface::{
        layer::Layer,
        parser::{eof_error, parser_rrc::RRCParser, time_to_milliseconds},
    },
};

/// Function that parses one log
/// # Errors
/// Return an error if the parsing fails
pub fn parse_one_block(lines: &[String], ix: &mut usize) -> Result<Trace, TramexError> {
    // no more lines to read
    if lines.is_empty() {
        return Err(eof_error(*ix as u64));
    }
    let mut start_line = 0;
    let mut end_line = 0;
    let mut should_stop = false;
    for one_line in lines.iter() {
        end_line += 1;
        if one_line.starts_with('#') {
            start_line += 1;
            continue;
        } else if one_line.starts_with(' ') || one_line.starts_with('\t') || one_line.trim().is_empty() {
            continue;
        } else {
            if should_stop {
                break;
            }
            should_stop = true;
        }
    }
    if end_line == 1 {
        return Err(eof_error(*ix as u64));
    }
    let lines_to_parse = &lines[start_line..end_line];
    let copy_ix = *ix + 1;
    *ix += end_line - 1;
    match lines_to_parse.first() {
        Some(first_line) => {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.is_empty() {
                return Err(TramexError::new(
                    format!("Not enough parts in the line {:?} (line {})", first_line, *ix as u64 + 1),
                    ErrorCode::FileParsing,
                ));
            }
            let date = match chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f") {
                Ok(rdate) => rdate,
                Err(_) => {
                    return Err(TramexError::new(
                        format!(
                            "Error while parsing date {:?} in {:?} (line {})",
                            parts[0], first_line, copy_ix
                        ),
                        ErrorCode::FileParsing,
                    ));
                }
            };
            let res_layer = Layer::from_str(parts[1].trim_start_matches('[').trim_end_matches(']'));
            let res_parse = match res_layer {
                Ok(Layer::RRC) => RRCParser::parse(lines_to_parse),
                _ => {
                    return Err(TramexError::new(
                        format!("Unknown message type {:?} in {:?} (line {})", parts[1], first_line, copy_ix),
                        ErrorCode::FileParsing,
                    ));
                }
            };
            match res_parse {
                Ok(mut trace) => {
                    trace.timestamp = time_to_milliseconds(&date) as u64;
                    Ok(trace)
                }
                Err(err) => Err(parsing_error_to_tramex_error(err, copy_ix as u64)),
            }
        }
        None => Err(eof_error(copy_ix as u64)),
    }
}
