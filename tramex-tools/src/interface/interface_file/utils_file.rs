//! utils functions for file interface

use std::str::FromStr;

use crate::{
    data::Trace,
    errors::TramexError,
    interface::{interface_file::parser::time_to_milliseconds, layer::Layer},
};

use super::{
    parser::{eof_error, parsing_error},
    parser_rrc::RRCParser,
};

/// Function that parses one log
/// # Errors
/// Return an error if the parsing fails
pub fn parse_one_block(lines: &Vec<&str>, ix: &mut usize) -> Result<Trace, Option<TramexError>> {
    // no more lines to read
    if *ix >= lines.len() {
        return Err(None);
    }
    let mut end_line = *ix;
    let mut should_stop = false;
    for one_line in lines[*ix..].iter() {
        if one_line.starts_with('#') || one_line.starts_with(' ') {
            end_line += 1;
            continue;
        } else {
            end_line += 1;
            if should_stop {
                break;
            }
            should_stop = true;
        }
    }
    let lines_to_parse = &lines[*ix..end_line];
    *ix = end_line - 1;
    match lines_to_parse.first() {
        Some(first_line) => {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            let date = match chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f") {
                Ok(rdate) => rdate,
                Err(_) => {
                    return Err(Some(parsing_error("Error while parsing date".to_string())));
                }
            };
            use super::parser::FileParser;
            let res_layer = Layer::from_str(parts[1].trim_start_matches('[').trim_end_matches(']'));
            let res_parse = match res_layer {
                Ok(Layer::RRC) => RRCParser::parse(lines_to_parse),
                _ => {
                    return Err(Some(parsing_error("Unknown message type".to_string())));
                }
            };
            match res_parse {
                Ok(mut trace) => {
                    *ix = end_line;
                    trace.timestamp = time_to_milliseconds(&date) as u64;
                    Ok(trace)
                }
                Err(err) => Err(err),
            }
        }
        None => Err(Some(eof_error())),
    }
}
