//! This module contains the data structures used to store the data of the application.
use crate::interface::{
    parse_config::FileMetadata, 
    layer::Layer, 
    parser::{parser_rrc::RRCInfos, parser_nas::NASInfos, parser_ngap::NGAPInfos, parser_gtpu::GTPUInfos},
    types::Direction,
};
use core::fmt::Debug;
use crate::asn1_parser::parse_asn1_to_json;

#[derive(Debug)]
/// Data structure to store Trace of the application.
pub struct Data {
    /// Vector of Trace.
    pub events: Vec<Trace>,
    /// Current index of the vector.
    pub current_index: usize,
    /// File metadata (connection type, version, etc.)
    pub metadata: FileMetadata,
}

impl Data {
    /// return the current trace
    pub fn get_current_trace(&self) -> Option<&Trace> {
        self.events.get(self.current_index)
    }

    /// return if the index is different from the current index
    pub fn is_different_index(&self, index: usize) -> bool {
        if index == 0 {
            return true;
        }
        self.current_index != index
    }

    /// clear the data
    pub fn clear(&mut self) {
        self.events.clear();
        self.current_index = 0;
        self.metadata = FileMetadata::default();
    }
}

impl Default for Data {
    fn default() -> Self {
        let default_data_size = 2048;
        Self {
            events: Vec::with_capacity(default_data_size),
            current_index: 0,
            metadata: FileMetadata::default(),
        }
    }
}

#[derive(Debug, Clone)]
/// Data structure to store Trace of the application.
pub struct Trace {
    /// Message type.
    /// Timestamp of the message.
    pub timestamp: i64,

    /// Layer of the message.
    pub layer: Layer,

    /// Message type.
    pub additional_infos: AdditionalInfos,

    /// Text representation of the message from the API
    pub text: Option<Vec<String>>,
}

impl Trace {
    /// Parse ASN.1 text from RRC messages and return as JSON
    /// 
    /// # Returns
    /// * `Some(Value)` - Parsed JSON if the trace has ASN.1 text and is an RRC layer
    /// * `None` - If no text available or not an RRC layer
    /// 
    /// # Errors
    /// Logs error if parsing fails but returns None
    pub fn parse_asn1_to_json(&self) -> Option<serde_json::Value> {
        // Only parse RRC layers
        if !matches!(self.layer, Layer::RRC) {
            return None;
        }
        
        // Check if we have text to parse
        let text = self.text.as_ref()?;
        
        // Find the start of ASN.1 structure (first line starting with '{')
        // Skip header lines and hex dump
        let asn1_lines: Vec<&String> = text.iter()
            .skip_while(|line| {
                let trimmed = line.trim();
                // Skip until we find a line that starts with '{'
                !trimmed.starts_with('{')
            })
            .collect();
        
        if asn1_lines.is_empty() {
            log::debug!("No ASN.1 structure found in text");
            return None;
        }
        
        // Join the ASN.1 lines
        let asn1_text: String = asn1_lines.iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        
        // Parse ASN.1 to JSON
        match parse_asn1_to_json(&asn1_text) {
            Ok(json) => {
                // log::debug!("Parsed ASN.1 to JSON: {}", serde_json::to_string_pretty(&json).unwrap_or_default());
                Some(json)
            }
            Err(e) => {
                log::warn!("Failed to parse ASN.1: {}", e);
                None
            }
        }
    }
}

/// Data structure to store custom messages (from the amarisoft API)
#[derive(Debug, Clone)]
pub enum AdditionalInfos {
    /// RRC message
    RRCInfos(RRCInfos),
    /// NAS message
    NASInfos(NASInfos),
    /// NGAP message
    NGAPInfos(NGAPInfos),
    /// GTPU message
    GTPUInfos(GTPUInfos),
    /// No additional info (for simple log entries like PHY, MAC, etc.)
    None,
}

impl AdditionalInfos {
    /// Get direction from additional infos
    pub fn get_direction(&self) -> Option<Direction> {
        match self {
            AdditionalInfos::RRCInfos(info) => Some(info.direction.clone()),
            AdditionalInfos::NASInfos(info) => Some(info.direction.clone()),
            AdditionalInfos::NGAPInfos(info) => Some(info.direction.clone()),
            AdditionalInfos::GTPUInfos(info) => Some(info.direction.clone()),
            AdditionalInfos::None => None,
        }
    }
    
    /// Get message name from additional infos
    pub fn get_message_name(&self) -> Option<String> {
        match self {
            AdditionalInfos::RRCInfos(info) => Some(info.canal_msg.clone()),
            AdditionalInfos::NASInfos(info) => Some(info.message_type.clone()),
            AdditionalInfos::NGAPInfos(info) => Some(info.message_type.clone()),
            AdditionalInfos::GTPUInfos(info) => Some(info.message_type.clone()),
            AdditionalInfos::None => None,
        }
    }
}
