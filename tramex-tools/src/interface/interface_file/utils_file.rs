//! utils functions for file interface
use std::str::FromStr;

use chrono::NaiveTime;
use chrono::Timelike;

use crate::{
    data::{MessageType, Trace},
    errors::TramexError,
    interface::{functions::extract_hexe, layer::Layer, types::Direction},
};

/// Convert a time to milliseconds.
pub fn time_to_milliseconds(time: &NaiveTime) -> i64 {
    let hours_in_ms = time.hour() as i64 * 3_600_000;
    let minutes_in_ms = time.minute() as i64 * 60_000;
    let seconds_in_ms = time.second() as i64 * 1000;
    let milliseconds = time.nanosecond() as i64 / 1_000_000; // convert nanoseconds to milliseconds

    hours_in_ms + minutes_in_ms + seconds_in_ms + milliseconds
}

/// Function that parses the first line of a log
/// # Errors
/// Return an error if the parsing fails
pub fn parse_line(line: &str) -> Result<MessageType, TramexError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(parsing_error("Could not find enough (5) parameters".to_string()));
    }
    let date = match chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f") {
        Ok(rdate) => rdate,
        Err(_) => {
            return Err(parsing_error("Error while parsing date".to_string()));
        }
    };
    let layer_result: Result<Layer, ()> = Layer::from_str(parts[1].trim_start_matches('[').trim_end_matches(']'));
    let direction_result = Direction::from_str(parts[2]);
    let binding: String = parts[5..].join(" ");
    let concatenated: Vec<&str> = binding.split(':').collect();
    let layer: Layer = match layer_result {
        Ok(l) => l,
        Err(_) => {
            return Err(parsing_error("The layer could not be parsed".to_string()));
        }
    };
    log::debug!("{:?}", layer);
    if layer == Layer::None {
        return Err(parsing_error("The layer could not be parsed".to_string()));
    }
    let direction = match direction_result {
        Ok(d) => d,
        Err(_) => return Err(parsing_error("The direction could not be parsed".to_string())),
    };
    if concatenated.len() < 2 || concatenated[0].is_empty() || concatenated[1].is_empty() {
        return Err(parsing_error(
            "The canal and/or canal message could not be parsed".to_string(),
        ));
    }
    return Ok(MessageType {
        timestamp: time_to_milliseconds(&date) as u64,
        layer,
        direction,
        canal: concatenated[0].to_owned(),
        canal_msg: concatenated[1].trim_start().to_owned(),
    });
}

/// Build a parsing error
pub fn parsing_error(message: String) -> TramexError {
    TramexError::new(message, crate::errors::ErrorCode::FileParsing)
}
/// Build a eof_error
pub fn eof_error() -> TramexError {
    TramexError::new("End of file".to_string(), crate::errors::ErrorCode::EndOfFile)
}

/// Function that parses one log
/// # Errors
/// Return an error if the parsing fails
pub fn parse_bloc(lines: &Vec<&str>, ix: &mut usize) -> Result<Trace, Option<TramexError>> {
    let lines_len = lines.len();
    if (lines_len as i32 - *ix as i32) < 3 {
        return Err(None);
    }
    let mtype = match parse_line(lines[*ix]) {
        Ok(m) => m,
        Err(e) => {
            return Err(Some(e));
        }
    };
    *ix += 1;
    let mut hex_str: Vec<&str> = vec![];
    while *ix < lines_len {
        match lines[*ix].trim_start().chars().next() {
            Some(c) => {
                if c == '{' {
                    break;
                }
            }
            None => {
                break;
            }
        }
        hex_str.push(lines[*ix]);
        *ix += 1;
    }
    if *ix >= lines_len {
        *ix -= 1;
        return Err(Some(parsing_error("Could not find the end of the hexadecimal".to_string())));
    }
    let hex = match extract_hexe(&hex_str) {
        Ok(h) => h,
        Err(e) => return Err(Some(e)),
    };

    let mut end = false;
    let mut brackets: i16 = 0;
    let start_block = *ix;
    while (*ix < lines_len) && !end {
        brackets += count_brackets(lines[*ix]);
        *ix += 1;
        if brackets == 0 {
            end = true;
        }
    }
    if *ix >= lines_len && !end {
        *ix -= 1;
        return Err(Some(parsing_error(
            "Could not parse the JSON like part, missing closing }".to_string(),
        )));
    }
    let trace = Trace {
        trace_type: mtype,
        hexa: hex,
        text: Some(lines[start_block..*ix].iter().map(|&s| s.to_string()).collect()),
    };
    *ix += 1;
    Ok(trace)
}

/// Counting Brackets
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
