//! Basic parser for simple layers (PHY, RLC, MAC, PDCP, SDAP, etc.)
use super::ParsingError;
use crate::data::{AdditionalInfos, Trace};
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
            return Err(ParsingError::new(
                format!("{:?} parser received empty lines", layer),
                0,
            ));
        }
        
        let text = lines.to_vec();
        
        let trace = Trace {
            timestamp: 0,
            layer,
            additional_infos: AdditionalInfos::None,
            text: Some(text),
        };
        Ok(trace)
    }
}
