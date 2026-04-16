//! Basic parser for simple layers (PHY, RLC, MAC, PDCP, SDAP, etc.)
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::data::{AdditionalInfos, Trace};
use crate::interface::association::TraceRelation;
use crate::interface::layer::Layer;

/// Basic Parser for simple layers (PHY, RLC, MAC, PDCP, SDAP, etc.)
/// This parser handles single-line logs with no complex structure
pub struct BasicParser;

impl BasicParser {
    /// Parse a log with a specific layer type
    /// # Errors
    /// Return an error if the parsing fails
    pub fn parse_with_layer(lines: &[String], layer: Layer) -> Result<Trace, ParsingError> {
        if lines.is_empty() {
            return Err(ParsingError::new(format!("{:?} parser received empty lines", layer), 0));
        }

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
            layer,
            additional_infos: AdditionalInfos::None,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}
