//! Parser for file interface

pub mod parser_rrc;

use crate::data::AdditionalInfos;
use crate::data::Trace;

use crate::errors::TramexError;
use chrono::NaiveTime;
use chrono::Timelike;

/// Build a parsing error
pub fn parsing_error(message: String, line_idx: u64) -> TramexError {
    TramexError::new(
        format!("{} (line {})", message, line_idx),
        crate::errors::ErrorCode::FileParsing,
    )
}

/// Trait for file parser
pub trait FileParser {
    /// Function that parses the first line of a log
    /// # Errors
    /// Return an error if the parsing fails
    fn parse_first_line(line: &str) -> Result<AdditionalInfos, TramexError>;

    /// Parse the lines of a file
    /// # Errors
    /// Return an error if the parsing fails
    fn parse(lines: &[&str]) -> Result<Trace, TramexError>;
}

/// Convert a time to milliseconds.
pub fn time_to_milliseconds(time: &NaiveTime) -> i64 {
    let hours_in_ms = time.hour() as i64 * 3_600_000;
    let minutes_in_ms = time.minute() as i64 * 60_000;
    let seconds_in_ms = time.second() as i64 * 1000;
    let milliseconds = time.nanosecond() as i64 / 1_000_000; // convert nanoseconds to milliseconds

    hours_in_ms + minutes_in_ms + seconds_in_ms + milliseconds
}

/// Build a eof_error
pub fn eof_error() -> TramexError {
    TramexError::new("End of file".to_string(), crate::errors::ErrorCode::EndOfFile)
}
