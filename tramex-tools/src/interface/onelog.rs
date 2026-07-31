//! This module contains the definition of the OneLog struct.

use std::str::FromStr;

use crate::data::Trace;
use crate::errors::TramexError;
use crate::interface::functions::extract_hexe;
use crate::interface::parser::ParsedHeader;

use crate::interface::{layer::Layer, types::SourceLog};
use crate::tramex_error;

use super::types::Direction;

#[derive(serde::Deserialize, Debug)]
/// Data structure to store the log.
pub struct OneLog {
    /// Each item is a string representing a line of log.
    pub data: Vec<String>,

    /// Milliseconds since January 1st 1970.
    pub timestamp: i64,

    /// log layer
    pub layer: Layer,

    /// Source of the log.
    pub src: SourceLog,

    /// index
    pub idx: u64,

    /// Direction (UL/DL/TO/FROM)
    pub dir: Option<String>,

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

    /// Log level
    pub level: Option<u8>,
}

impl OneLog {
    /// Extract the hexadecimal representation of the log.
    /// # Errors
    /// Returns a TramexError if the hexe representation could not be extracted.
    pub fn extract_hexe(&self) -> Result<Vec<u8>, TramexError> {
        extract_hexe(&self.data)
    }

    /// Extract the canal message of the log.
    pub fn extract_canal_msg(&self) -> Option<String> {
        // TODO implement this function correctly
        if let Some(data_line) = self.data.first() {
            log::debug!("{data_line:?}");
            return Some(data_line.to_owned());
        }
        None
    }

    /// Build a ParsedHeader from the WebSocket JSON fields.
    fn build_header(&self) -> Result<ParsedHeader, TramexError> {
        let direction = match &self.dir {
            Some(opt_dir) => Direction::from_str(opt_dir).unwrap_or(Direction::NA),
            None => Direction::NA,
        };

        Ok(ParsedHeader {
            timestamp: self.timestamp,
            layer: self.layer.clone(),
            direction,
            ue_id: self.ue_id,
            cell: self.cell,
            rnti: self.rnti,
            frame: self.frame,
            slot: self.slot,
            channel: self.channel.clone(),
            connection_info: None,
        })
    }

    /// Extract the data of the log using the unified LayerParser pipeline.
    /// # Errors
    /// Returns a TramexError if the data could not be extracted.
    pub fn extract_data(&self) -> Result<Trace, TramexError> {
        use crate::interface::parser::build_trace;
        use crate::interface::parser::LayerParser;
        use crate::interface::parser::parser_basic::BasicParser;
        use crate::interface::parser::parser_gtpu::GTPUParser;
        use crate::interface::parser::parser_nas::NASParser;
        use crate::interface::parser::parser_ngap::NGAPParser;
        use crate::interface::parser::parser_phy::PHYParser;
        use crate::interface::parser::parser_rrc::RRCParser;

        let header = self.build_header()?;

        let additional_infos = match self.layer {
            Layer::RRC => RRCParser::parse_layer(&header, &self.data),
            Layer::NAS => NASParser::parse_layer(&header, &self.data),
            Layer::NGAP => NGAPParser::parse_layer(&header, &self.data),
            Layer::GTPU => GTPUParser::parse_layer(&header, &self.data),
            Layer::PHY => PHYParser::parse_layer(&header, &self.data),
            _ => BasicParser::parse_layer(&header, &self.data),
        }
        .map_err(|e| tramex_error!(e.message, crate::errors::ErrorCode::ParsingLayerNotImplemented))?;

        Ok(build_trace(&header, additional_infos, &self.data))
    }
}
