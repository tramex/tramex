//! Parser for file interface

pub mod hex_extractor;
pub mod parser_basic;
pub mod parser_gtpu;
pub mod parser_nas;
pub mod parser_ngap;
pub mod parser_phy;
pub mod parser_rrc;

use crate::data::AdditionalInfos;
use crate::data::Trace;

use crate::errors::ErrorCode;
use crate::errors::TramexError;
use crate::interface::layer::Layer;
use crate::interface::types::Direction;
use crate::tramex_error;
use chrono::NaiveTime;
use chrono::Timelike;
use std::str::FromStr;

/// Common metadata extracted from either a file header line or WebSocket JSON fields.
/// This serves as the unified input to all layer parsers.
#[derive(Debug, Clone, Default)]
pub struct ParsedHeader {
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Protocol layer
    pub layer: Layer,
    /// Direction (UL/DL/TO/FROM)
    pub direction: Direction,
    /// UE identifier
    pub ue_id: Option<u64>,
    /// Cell identifier
    pub cell: Option<u64>,
    /// RNTI
    pub rnti: Option<u64>,
    /// Frame number (PHY)
    pub frame: Option<u16>,
    /// Slot number (PHY)
    pub slot: Option<u8>,
    /// Channel name (PHY: PDCCH, PDSCH, etc.)
    pub channel: Option<String>,
    /// Connection info (NGAP/GTPU: IP:port)
    pub connection_info: Option<String>,
}

impl ParsedHeader {
    /// Build a ParsedHeader by parsing the first line of a file-format trace.
    /// File format: "HH:MM:SS.mmm [LAYER] DIR id1 id2 id3 ... payload"
    ///
    /// # Errors
    /// Returns a ParsingError if the line cannot be parsed.
    pub fn from_file_line(first_line: &str) -> Result<Self, ParsingError> {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(ParsingError::new(format!("Not enough parts in line: {}", first_line), 0));
        }

        let timestamp = parse_timestamp(first_line)?;

        let layer = Layer::from_str(parts[1].trim_start_matches('[').trim_end_matches(']'))
            .map_err(|_| ParsingError::new(format!("Unknown layer: {}", parts[1]), 0))?;

        let direction = Direction::from_str(parts[2]).unwrap_or(Direction::NA);

        // For NGAP/GTPU, extract connection_info from the parts
        let connection_info = match layer {
            Layer::NGAP => {
                if parts.len() > 5 && parts[5].contains(':') {
                    Some(parts[5].to_string())
                } else {
                    None
                }
            }
            Layer::GTPU => {
                if parts.len() > 3 && parts[3].contains(':') {
                    Some(parts[3].to_string())
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(Self {
            timestamp,
            layer,
            direction,
            ue_id: None,
            cell: None,
            rnti: None,
            frame: None,
            slot: None,
            channel: None,
            connection_info,
        })
    }
}

/// Parsing error
#[derive(Debug)]
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
    tramex_error!(format!("{} (line {})", error.message, index), ErrorCode::FileParsing)
}

/// Trait for file parser (legacy — kept for backward compatibility)
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

/// Unified layer parser trait.
/// Takes a ParsedHeader (built from either file or WebSocket) plus payload data lines
/// and produces AdditionalInfos for that layer.
pub trait LayerParser {
    /// Parse payload data lines given the pre-extracted header metadata.
    /// # Errors
    /// Return an error if the parsing fails
    fn parse_layer(header: &ParsedHeader, data_lines: &[String]) -> Result<AdditionalInfos, ParsingError>;
}

/// Extract the payload portion of file-format lines by stripping the header from the first line.
/// For a file line like "13:20:58.310 [RRC] DL 0001 01 4601  DCCH-NR: RRC release",
/// this returns the part after the header prefix that is layer-specific payload.
pub fn extract_file_payload(first_line: &str, layer: &Layer) -> String {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    // The payload start index depends on the layer format:
    // RRC:  "TIME [RRC] DIR ID1 ID2 ID3 CANAL: MSG" → parts[5..]
    // NAS:  "TIME [NAS] DIR ID1 PROTO: MSG" → parts[4..]
    // NGAP: "TIME [NGAP] DIR ID1 ID2 IP:PORT MSG..." → parts[6..] (or 5 if no connection)
    // GTPU: "TIME [GTPU] DIR IP:PORT MSG..." → parts[4..]
    // PHY:  entire line is needed for parse_phy_lines
    match layer {
        Layer::RRC => {
            if parts.len() > 5 {
                parts[5..].join(" ")
            } else {
                first_line.to_string()
            }
        }
        Layer::NAS => {
            if parts.len() > 4 {
                parts[4..].join(" ")
            } else {
                first_line.to_string()
            }
        }
        Layer::NGAP => {
            // Skip: TIME [NGAP] DIR ID1 ID2 IP:PORT → payload starts at 6
            let start = if parts.len() > 5 && parts[5].contains(':') { 6 } else { 5 };
            if parts.len() > start {
                parts[start..].join(" ")
            } else {
                first_line.to_string()
            }
        }
        Layer::GTPU => {
            // Skip: TIME [GTPU] DIR IP:PORT → payload starts at 4
            let start = if parts.len() > 3 && parts[3].contains(':') { 4 } else { 3 };
            if parts.len() > start {
                parts[start..].join(" ")
            } else {
                first_line.to_string()
            }
        }
        _ => first_line.to_string(),
    }
}

/// Build a Trace from a ParsedHeader, AdditionalInfos, and data lines.
/// This is the single assembly point used by both file and WebSocket paths.
pub fn build_trace(header: &ParsedHeader, additional_infos: AdditionalInfos, data_lines: &[String]) -> Trace {
    use crate::interface::association::TraceRelation;
    let binary = hex_extractor::extract_binary_from_lines(data_lines);
    Trace {
        timestamp: header.timestamp,
        layer: header.layer.clone(),
        additional_infos,
        text: Some(data_lines.to_vec()),
        binary,
        relation: TraceRelation::default(),
    }
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

/// Parse timestamp from the first line of a trace
/// Expected format: "HH:MM:SS.mmm [LAYER] ..."
/// # Errors
/// Returns ParsingError if timestamp cannot be parsed
pub fn parse_timestamp(first_line: &str) -> Result<i64, ParsingError> {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.is_empty() {
        return Err(ParsingError::new("Empty line, cannot parse timestamp".to_string(), 0));
    }

    let date = chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f")
        .map_err(|_| ParsingError::new(format!("Error parsing timestamp '{}' in line: {}", parts[0], first_line), 0))?;

    Ok(time_to_milliseconds(&date))
}

/// Build a eof_error
#[inline]
pub fn eof_error(line_idx: u64) -> TramexError {
    tramex_error!(
        format!("End of file (line {})", line_idx),
        crate::errors::ErrorCode::EndOfFile
    )
}
