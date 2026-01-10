//! This module contains the data structures used to store the data of the application.
use crate::interface::association::{
    TraceRelation, AssociationStatus, AssociationRules, TraceMatcher,
};
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

    /// Compute parent association for a trace at the given index using lazy evaluation.
    /// If already computed, returns the cached result. Otherwise, computes and caches it.
    /// 
    /// # Arguments
    /// * `index` - Index of the trace to compute parent for
    /// * `rules` - The association rules to use for matching
    /// 
    /// # Returns
    /// * The parent index if found, None otherwise
    pub fn compute_parent(&mut self, index: usize, rules: &AssociationRules) -> Option<usize> {
        // Check if already computed
        if let Some(trace) = self.events.get(index) {
            if trace.relation.parent.is_computed() {
                return trace.relation.get_parent_index();
            }
        }

        // Get the trace's layer to find applicable rules
        let layer = match self.events.get(index) {
            Some(t) => t.layer.clone(),
            None => return None,
        };

        // Try each applicable rule
        let applicable_rules = rules.rules_for_layer(&layer);
        for rule in applicable_rules {
            let status = TraceMatcher::find_relative(index, &self.events, rule, &layer, &rule.target_layer());
            
            // Update the trace's relation
            if let Some(trace) = self.events.get_mut(index) {
                trace.relation.parent = status.clone();
            }

            // If we found a parent, also set the child relation on the parent
            if let AssociationStatus::Found(parent_idx) = status {
                if let Some(parent_trace) = self.events.get_mut(parent_idx) {
                    parent_trace.relation.set_child(index);
                }
                return Some(parent_idx);
            }
        }

        // Mark as not found if no rules matched
        if let Some(trace) = self.events.get_mut(index) {
            if !trace.relation.parent.is_computed() {
                trace.relation.set_parent_not_applicable();
            }
        }

        None
    }

    /// Compute all associations for all traces in the events vector.
    /// This applies all rules to each trace automatically:
    /// - For each trace, finds rules where the trace's layer is the source layer
    /// - Computes the relationship and updates both parent (on source) and child (on target)
    /// 
    /// After calling this method, you can use `trace.relation.get_parent_index()` or 
    /// `trace.relation.get_child_index()` to get related traces.
    pub fn compute_all_associations(&mut self, rules: &AssociationRules) {
        crate::interface::association::compute_associations(&mut self.events, rules, 0);
    }

    /// Get the parent trace for a trace at the given index.
    /// This will compute the association if not already done.
    /// 
    /// # Arguments
    /// * `index` - Index of the trace
    /// * `rules` - The association rules to use
    /// 
    /// # Returns
    /// * Reference to the parent trace if found
    pub fn get_parent_trace(&mut self, index: usize, rules: &AssociationRules) -> Option<&Trace> {
        let parent_idx = self.compute_parent(index, rules)?;
        self.events.get(parent_idx)
    }

    /// Get the child trace for a trace at the given index.
    /// Note: child relation is set when the child's parent is computed.
    /// 
    /// # Arguments
    /// * `index` - Index of the trace
    /// 
    /// # Returns
    /// * Reference to the child trace if found
    pub fn get_child_trace(&self, index: usize) -> Option<&Trace> {
        let trace = self.events.get(index)?;
        let child_idx = trace.relation.get_child_index()?;
        self.events.get(child_idx)
    }

    /// Invalidate all associations (useful when new traces are added)
    pub fn invalidate_associations(&mut self) {
        for trace in &mut self.events {
            trace.relation.invalidate();
        }
    }

    /// Invalidate associations in a range around newly added traces
    /// This is more efficient than invalidating all associations
    /// 
    /// # Arguments
    /// * `start_index` - Start of the range where new traces were added
    /// * `window` - Window size to invalidate around the new traces
    pub fn invalidate_associations_in_range(&mut self, start_index: usize, window: usize) {
        let start = start_index.saturating_sub(window);
        let end = (start_index + window).min(self.events.len());
        
        for trace in &mut self.events[start..end] {
            trace.relation.invalidate();
        }
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
    /// Timestamp of the message.
    pub timestamp: i64,

    /// Layer of the message.
    pub layer: Layer,

    /// Additional layer-specific information.
    pub additional_infos: AdditionalInfos,

    /// Text representation of the message from the API
    pub text: Option<Vec<String>>,

    /// Binary representation extracted from hex dump (if present and complete)
    pub binary: Option<Vec<u8>>,

    /// Parent/child relationship with other traces
    pub relation: TraceRelation,
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
