//! Association rules for matching related traces

use crate::data::Trace;
use crate::interface::layer::Layer;
use crate::interface::types::Direction;

/// Preferred search direction when looking for related traces
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SearchDirection {
    /// Search backward first (previous traces), then forward
    #[default]
    BackwardFirst,
    /// Search forward first (next traces), then backward  
    ForwardFirst,
    /// Only search backward
    BackwardOnly,
    /// Only search forward
    ForwardOnly,
}

impl SearchDirection {
    /// Determine preferred direction based on trace direction
    /// UL messages: parent likely before (decapsulation happened)
    /// DL messages: parent likely after (encapsulation will happen)
    pub fn from_direction(direction: &Direction) -> Self {
        match direction {
            Direction::UL | Direction::FROM => SearchDirection::BackwardFirst,
            Direction::DL | Direction::TO => SearchDirection::ForwardFirst,
            Direction::NA => SearchDirection::BackwardFirst,
        }
    }
}

/// Trait for defining association rules between trace types
pub trait AssociationRule: Send + Sync {
    /// The layer this rule applies to (source layer when searching)
    fn source_layer(&self) -> Layer;

    /// The layer we're searching for (target layer)
    fn target_layer(&self) -> Layer;

    /// Check if two traces are related according to this rule.
    ///
    /// # Arguments
    /// * `trace` - The source trace
    /// * `candidate` - A potential related trace to check
    ///
    /// # Returns
    /// * `true` if the traces are related
    fn matches(&self, trace: &Trace, candidate: &Trace) -> bool;

    /// Get the preferred search direction based on the source trace
    fn preferred_direction(&self, source: &Trace) -> SearchDirection {
        source
            .additional_infos
            .get_direction()
            .map(|d| SearchDirection::from_direction(&d))
            .unwrap_or_default()
    }

    /// Get the window size (max number of traces to search in each direction)
    fn window_size(&self) -> usize {
        10 // Default window size for searching related traces
    }

    /// Filter for valid source message names (if empty, all messages are valid)
    /// This is checked before attempting to match
    fn valid_source_messages(&self) -> &[&str] {
        &[] // Empty means all source messages are valid
    }

    /// Filter for valid target message names (if empty, all messages are valid)
    /// This is checked before attempting to match
    fn valid_target_messages(&self) -> &[&str] {
        &[] // Empty means all target messages are valid
    }

    /// Returns true if the source trace is the child in the relationship
    /// - true: source is child, target is parent (e.g., NAS→RRC where NAS is child)
    /// - false: source is parent, target is child (e.g., NGAP→NAS where NAS is child)
    fn source_is_child(&self) -> bool {
        true // Default: source trace is the child
    }
}

/// Collection of all available association rules
pub struct AssociationRules {
    /// Rules
    rules: Vec<Box<dyn AssociationRule>>,
}

impl Default for AssociationRules {
    fn default() -> Self {
        Self::new()
    }
}

impl AssociationRules {
    /// Create a new rule set with default rules
    pub fn new() -> Self {
        let rules: Vec<Box<dyn AssociationRule>> = vec![Box::new(NasToRrcRule::new()), Box::new(NgapToNasRule::new())];
        Self { rules }
    }

    /// Get all rules that apply to a given source layer
    pub fn rules_for_layer(&self, layer: &Layer) -> Vec<&dyn AssociationRule> {
        self.rules
            .iter()
            .filter(|r| &r.source_layer() == layer)
            .map(|r| r.as_ref())
            .collect()
    }
}

// Concrete rules

/// Rule for associating NAS messages with their parent RRC messages
#[derive(Debug, Default)]
pub struct NasToRrcRule {
    /// Number of bytes to match from the NAS message binary
    /// We match only the first 14 bytes because of potential ciphering differences
    pub byte_match_length: usize,
    /// Explicit list of RRC messages that can carry NAS
    pub valid_rrc_messages: &'static [&'static str],
    /// Explicit list of NAS messages that can be carried by RRC
    pub valid_nas_messages: &'static [&'static str],
}

impl NasToRrcRule {
    /// Create a new NAS to RRC rule
    pub fn new() -> Self {
        Self {
            byte_match_length: 14, // Match first 14 bytes (does not need full match due to ciphering)
            valid_rrc_messages: &[
                "dl information transfer",
                "ul information transfer",
                "rrc setup complete",
                "rrc reconfiguration",
            ], // All RRC messages that can carry NAS
            valid_nas_messages: &["service request", "registration accept", "deregistration request"], // All NAS messages that can be carried by RRC
        }
    }

    /// Extract dedicated NAS message binary from RRC trace text
    /// Looks for dedicatedNAS-Message field in ASN.1 structure and extracts the hex value
    fn extract_rrc_dedicated_nas_binary(&self, trace: &Trace) -> Option<Vec<u8>> {
        let text = trace.text.as_ref()?;

        for line in text.iter() {
            let trimmed = line.trim();

            // Look for dedicatedNAS-Message field in ASN.1 structure
            // The hex value is typically on the same line: "dedicatedNAS-Message '7E026ECD92EF637E0043'H"
            if trimmed.contains("dedicatedNAS-Message") || trimmed.contains("nas-MessageContainer") {
                // Extract hex value from the same line
                // Format: "'7E026ECD92EF637E0043'H" or similar
                if let Some(hex_start) = trimmed.find('\'')
                    && let Some(hex_end) = trimmed[hex_start + 1..].find('\'')
                {
                    let hex_str = &trimmed[hex_start + 1..hex_start + 1 + hex_end];
                    // Convert hex string to bytes
                    return Self::hex_string_to_bytes(hex_str);
                }
            }
        }
        None
    }

    /// Convert hex string to bytes
    fn hex_string_to_bytes(hex_str: &str) -> Option<Vec<u8>> {
        let hex_clean: String = hex_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        if !hex_clean.len().is_multiple_of(2) {
            return None;
        }

        let mut bytes = Vec::new();
        for i in (0..hex_clean.len()).step_by(2) {
            if let Ok(byte) = u8::from_str_radix(&hex_clean[i..i + 2], 16) {
                bytes.push(byte);
            } else {
                return None;
            }
        }

        Some(bytes)
    }
}

impl AssociationRule for NasToRrcRule {
    fn source_layer(&self) -> Layer {
        Layer::NAS
    }

    fn target_layer(&self) -> Layer {
        Layer::RRC
    }

    fn valid_source_messages(&self) -> &[&str] {
        self.valid_nas_messages
    }

    fn valid_target_messages(&self) -> &[&str] {
        self.valid_rrc_messages
    }

    fn matches(&self, trace: &Trace, candidate: &Trace) -> bool {
        // Determine which trace is NAS and which is RRC (bidirectional matching)
        let (nas_trace, rrc_trace) = if trace.layer == Layer::NAS && candidate.layer == Layer::RRC {
            (trace, candidate)
        } else if trace.layer == Layer::RRC && candidate.layer == Layer::NAS {
            (candidate, trace)
        } else {
            return false;
        };

        // Check if RRC message type is a valid candidate for carrying NAS
        let rrc_msg_name = rrc_trace
            .additional_infos
            .get_message_name()
            .unwrap_or_default()
            .to_lowercase();

        let is_valid_candidate = self.valid_rrc_messages.iter().any(|&msg| rrc_msg_name == msg);

        if !is_valid_candidate {
            return false;
        }

        // Get NAS binary data
        let nas_binary = match &nas_trace.binary {
            Some(b) => b,
            None => return false,
        };

        // Extract dedicated NAS message binary from RRC trace
        let rrc_nas_binary = match self.extract_rrc_dedicated_nas_binary(rrc_trace) {
            Some(b) => b,
            None => return false,
        };

        // Compare first N bytes
        // We only match the first 14 bytes because the rest may differ due to ciphering
        let match_len = self.byte_match_length.min(nas_binary.len()).min(rrc_nas_binary.len());
        if match_len == 0 {
            return false;
        }

        // Binary comparison of first 14 bytes
        log::debug!("match found");
        nas_binary[..match_len] == rrc_nas_binary[..match_len]
    }
}

/// Rule for associating NGAP messages with their child NAS messages
#[derive(Debug, Default)]
pub struct NgapToNasRule {
    /// Number of bytes to match from the NAS message binary
    /// We match only the first 14 bytes because of potential ciphering differences
    pub byte_match_length: usize,
    /// Explicit list of NGAP messages that can carry NAS (source filter)
    pub valid_ngap_messages: &'static [&'static str],
    /// Explicit list of NAS messages that can be carried by NGAP (target filter)
    pub valid_nas_messages: &'static [&'static str],
}

impl NgapToNasRule {
    /// Create a new NGAP to NAS rule
    pub fn new() -> Self {
        Self {
            byte_match_length: 14, // Match first 14 bytes (does not need full match due to ciphering)
            valid_ngap_messages: &[
                "initial ue message",
                "downlink nas transport",
                "uplink nas transport",
                "initial context setup request",
            ], // NGAP messages that can carry NAS
            valid_nas_messages: &[
                "service request",
                "registration accept",
                "deregistration request",
                "ul nas transport",
                "dl nas transport",
            ], // NAS messages that can be carried by NGAP
        }
    }

    /// Extract NAS message binary from NGAP trace text
    /// Looks for id-NAS-PDU field in ASN.1 structure and extracts the hex value from the following value line
    ///
    /// Format example:
    /// ```text
    /// {
    ///   id id-NAS-PDU,
    ///   criticality reject,
    ///   value '7E017E49623C607E004509000BF200F110800101E6162E91'H
    /// },
    /// ```
    fn extract_ngap_nas_binary(&self, trace: &Trace) -> Option<Vec<u8>> {
        let text = trace.text.as_ref()?;

        let mut found_nas_pdu = false;

        for line in text.iter() {
            let trimmed = line.trim();

            // Look for id-NAS-PDU line
            if trimmed.contains("id-NAS-PDU") {
                found_nas_pdu = true;
                continue;
            }

            // After finding id-NAS-PDU, look for the value line with hex string
            if found_nas_pdu && trimmed.starts_with("value ") {
                // Extract hex value: value '7E017E...'H
                if let Some(hex_start) = trimmed.find('\'')
                    && let Some(hex_end) = trimmed[hex_start + 1..].find('\'')
                {
                    let hex_str = &trimmed[hex_start + 1..hex_start + 1 + hex_end];
                    return Self::hex_string_to_bytes(hex_str);
                }
                // Reset if value line didn't have hex (it's a different value field)
                found_nas_pdu = false;
            }

            // Reset if we hit a closing brace without finding the value
            if found_nas_pdu && trimmed.starts_with('}') {
                found_nas_pdu = false;
            }
        }
        None
    }

    /// Convert hex string to bytes
    fn hex_string_to_bytes(hex_str: &str) -> Option<Vec<u8>> {
        let hex_clean: String = hex_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();

        if !hex_clean.len().is_multiple_of(2) {
            return None;
        }

        let mut bytes = Vec::new();
        for i in (0..hex_clean.len()).step_by(2) {
            if let Ok(byte) = u8::from_str_radix(&hex_clean[i..i + 2], 16) {
                bytes.push(byte);
            } else {
                return None;
            }
        }

        Some(bytes)
    }
}

impl AssociationRule for NgapToNasRule {
    fn source_layer(&self) -> Layer {
        Layer::NGAP
    }

    fn target_layer(&self) -> Layer {
        Layer::NAS
    }

    fn valid_source_messages(&self) -> &[&str] {
        self.valid_ngap_messages
    }

    fn valid_target_messages(&self) -> &[&str] {
        self.valid_nas_messages
    }

    fn source_is_child(&self) -> bool {
        false // NGAP is parent, NAS is child
    }

    fn preferred_direction(&self, source: &Trace) -> SearchDirection {
        match source.additional_infos.get_direction() {
            Some(Direction::UL) | Some(Direction::TO) => SearchDirection::BackwardFirst,
            Some(_) => SearchDirection::ForwardFirst,
            None => SearchDirection::default(),
        }
    }

    fn matches(&self, trace: &Trace, candidate: &Trace) -> bool {
        // trace is NGAP (source), candidate is NAS (target)
        if trace.layer != Layer::NGAP || candidate.layer != Layer::NAS {
            return false;
        }
        let (ngap_trace, nas_trace) = (trace, candidate);

        // Get NAS binary data
        let nas_binary = match &nas_trace.binary {
            Some(b) => b,
            None => return false,
        };

        // Extract dedicated NAS message binary from NGAP trace
        let ngap_nas_binary = match self.extract_ngap_nas_binary(ngap_trace) {
            Some(b) => b,
            None => return false,
        };

        // Compare first N bytes
        // We only match the first 14 bytes because the rest may differ due to ciphering
        let match_len = self.byte_match_length.min(nas_binary.len()).min(ngap_nas_binary.len());
        if match_len == 0 {
            return false;
        }

        // Binary comparison of first 14 bytes
        log::debug!("match found");
        nas_binary[..match_len] == ngap_nas_binary[..match_len]
    }
}
