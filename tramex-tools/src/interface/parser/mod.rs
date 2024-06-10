//! Parser for file interface

pub mod parser_rrc;

use crate::data::AdditionalInfos;
use crate::data::Trace;

use crate::errors::ErrorCode;
use crate::errors::TramexError;
use chrono::NaiveTime;
use chrono::Timelike;

/// Parsing error
pub struct ParsingError {
    /// Error message
    pub message: String,

    /// Line index
    pub line_idx: u64,
}

impl ParsingError {
    /// Create a new parsing error
    pub fn new(message: String, line_idx: u64) -> Self {
        Self { message, line_idx }
    }
}

/// Convert a parsing error to a tramex error
#[inline]
pub fn parsing_error_to_tramex_error(error: ParsingError, idx: u64) -> TramexError {
    let index = idx + error.line_idx;
    TramexError::new(format!("{} (line {})", error.message, index), ErrorCode::FileParsing)
}

/// Trait for file parser
pub trait FileParser {
    /// Function that parses the first line of a log
    /// # Errors
    /// Return an error if the parsing fails
    fn parse_additional_infos(line: &[String]) -> Result<AdditionalInfos, ParsingError>;

    /// Parse the lines of a file
    /// # Errors
    /// Return an error if the parsing fails
    fn parse(lines: &[String]) -> Result<Trace, ParsingError>;
}

/// Convert a time to milliseconds.
#[inline]
pub fn time_to_milliseconds(time: &NaiveTime) -> i64 {
    let hours_in_ms = time.hour() as i64 * 3_600_000;
    let minutes_in_ms = time.minute() as i64 * 60_000;
    let seconds_in_ms = time.second() as i64 * 1000;
    let milliseconds = time.nanosecond() as i64 / 1_000_000; // convert nanoseconds to milliseconds

    hours_in_ms + minutes_in_ms + seconds_in_ms + milliseconds
}

/// Build a eof_error
#[inline]
pub fn eof_error(line_idx: u64) -> TramexError {
    TramexError::new(
        format!("End of file (line {})", line_idx),
        crate::errors::ErrorCode::EndOfFile,
    )
}
