//! Identity Panel
//! 
//! Displays parsed identity information from NAS messages (PDU session establishment, etc.)

use crate::event_system::{EventSubscriber, EventContext};
use egui::{self, RichText};
use crate::theme::ThemeColors;
use tramex_tools::{
    data::Trace,
    interface::{layer::Layer, parse_config::{FileMetadata, Technology}},
};
use crate::panels::PanelView;

/// QoS Rule information
#[derive(Clone, Debug, Default)]
pub struct QosRule {
    /// QoS rule identifier
    pub identifier: Option<String>,
    /// DQR (Default QoS Rule indicator)
    pub dqr: Option<String>,
    /// QoS rule precedence
    pub precedence: Option<String>,
    /// QoS Flow Identifier
    pub qfi: Option<String>,
}

/// Decoded TAI information
#[derive(Clone, Debug, Default)]
pub struct TaiInfo {
    /// Mobile Country Code
    pub mcc: Option<String>,
    /// Mobile Network Code
    pub mnc: Option<String>,
    /// Tracking Area Codes
    pub tacs: Vec<String>,
    /// Raw data (for undecoded types)
    pub raw_data: Option<String>,
}

/// Identity Panel - displays NAS identity information
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Identity {
    /// Current index
    #[serde(skip)]
    current_index: usize,
    
    /// Cached metadata for UI rendering
    #[serde(skip)]
    metadata: FileMetadata,
    
    
    // === 4G LTE specific fields ===
    
    /// 4G GUTI MCC
    #[serde(skip)]
    lte_guti_mcc: Option<String>,
    
    /// 4G GUTI MNC
    #[serde(skip)]
    lte_guti_mnc: Option<String>,
    
    /// 4G MME Group ID
    #[serde(skip)]
    lte_mme_group_id: Option<String>,
    
    /// 4G MME Code
    #[serde(skip)]
    lte_mme_code: Option<String>,
    
    /// 4G M-TMSI
    #[serde(skip)]
    lte_m_tmsi: Option<String>,
    
    /// 4G PDN IPv4 address
    #[serde(skip)]
    lte_pdn_ipv4: Option<String>,
    
    /// 4G PDN IPv6 address
    #[serde(skip)]
    lte_pdn_ipv6: Option<String>,
    
    /// 4G PDN type
    #[serde(skip)]
    lte_pdn_type: Option<String>,
    
    /// 4G Access Point Name
    #[serde(skip)]
    lte_apn: Option<String>,
    
    /// 4G TAI info
    #[serde(skip)]
    lte_tai_info: TaiInfo,
    
    /// 4G DNS IPv4
    #[serde(skip)]
    lte_dns_ipv4: Option<String>,
    
    /// 4G DNS IPv6
    #[serde(skip)]
    lte_dns_ipv6: Option<String>,

    // === 5G NR specific fields ===

    /// PDU IPv4 address
    #[serde(skip)]
    nr_pdu_ipv4: Option<String>,
    
    /// PDU IPv6 address
    #[serde(skip)]
    nr_pdu_ipv6: Option<String>,
    
    /// PDU session type
    #[serde(skip)]
    nr_pdu_session_type: Option<String>,
    
    /// DNN (Data Network Name)
    #[serde(skip)]
    nr_dnn: Option<String>,
    
    /// 5G-GUTI MCC
    #[serde(skip)]
    nr_guti_mcc: Option<String>,
    
    /// 5G-GUTI MNC
    #[serde(skip)]
    nr_guti_mnc: Option<String>,
    
    /// 5G-GUTI AMF Region ID
    #[serde(skip)]
    nr_guti_amf_region_id: Option<String>,
    
    /// 5G-GUTI AMF Set ID
    #[serde(skip)]
    nr_guti_amf_set_id: Option<String>,
    
    /// 5G-GUTI AMF Pointer
    #[serde(skip)]
    nr_guti_amf_pointer: Option<String>,
    
    /// 5G-GUTI 5G-TMSI
    #[serde(skip)]
    nr_guti_5g_tmsi: Option<String>,
    
    /// QoS Rules list
    #[serde(skip)]
    nr_qos_rules: Vec<QosRule>,
    
    /// 5QI (from QoS flow descriptions)
    #[serde(skip)]
    nr_fiveqi: Option<String>,
    
    /// DNS Server IPv4 Address
    #[serde(skip)]
    nr_dns_ipv4: Option<String>,
    
    /// TAI information
    #[serde(skip)]
    nr_tai_info: TaiInfo,
    
    /// Session AMBR downlink
    #[serde(skip)]
    nr_session_ambr_dl: Option<String>,
    
    /// Session AMBR uplink
    #[serde(skip)]
    nr_session_ambr_ul: Option<String>,
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

impl Identity {
    /// Create a new Identity panel
    pub fn new() -> Self {
        Self {
            current_index: 0,
            metadata: FileMetadata::default(),
            // 4G LTE fields
            lte_guti_mcc: None,
            lte_guti_mnc: None,
            lte_mme_group_id: None,
            lte_mme_code: None,
            lte_m_tmsi: None,
            lte_pdn_ipv4: None,
            lte_pdn_ipv6: None,
            lte_pdn_type: None,
            lte_apn: None,
            lte_tai_info: TaiInfo::default(),
            lte_dns_ipv4: None,
            lte_dns_ipv6: None,
            // 5G NR fields
            nr_pdu_ipv4: None,
            nr_pdu_ipv6: None,
            nr_pdu_session_type: None,
            nr_dnn: None,
            nr_guti_mcc: None,
            nr_guti_mnc: None,
            nr_guti_amf_region_id: None,
            nr_guti_amf_set_id: None,
            nr_guti_amf_pointer: None,
            nr_guti_5g_tmsi: None,
            nr_qos_rules: Vec::new(),
            nr_fiveqi: None,
            nr_dns_ipv4: None,
            nr_tai_info: TaiInfo::default(),
            nr_session_ambr_dl: None,
            nr_session_ambr_ul: None,
        }
    }
    
    /// Decode TAI list from hex data string
    /// Format: first byte is type (00, 01, 10, 11)
    /// For 00 & 01: bytes 2-4 are MCC/MNC in BCD, then TACs
    fn decode_tai_list(data: &str) -> TaiInfo {
        let bytes: Vec<u8> = data
            .split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        
        if bytes.is_empty() {
            return TaiInfo {
                raw_data: Some(data.to_string()),
                ..Default::default()
            };
        }
        
        let list_type = bytes[0] & 0x03; // First 2 bits
        
        // Only decode for type 00 and 01
        if list_type == 0b10 {
            // Type 10: not decoded, return raw
            return TaiInfo {
                raw_data: Some(data.to_string()),
                ..Default::default()
            };
        }
        
        if bytes.len() < 4 {
            return TaiInfo {
                raw_data: Some(data.to_string()),
                ..Default::default()
            };
        }
        
        // Decode MCC/MNC from bytes 2-4 (indices 1-3) in BCD format
        // Byte 1: MCC2 (high nibble), MCC1 (low nibble)
        // Byte 2: MNC3 (high nibble, F if 2-digit MNC), MCC3 (low nibble)
        // Byte 3: MNC2 (high nibble), MNC1 (low nibble)
        let mcc1 = bytes[1] & 0x0F;
        let mcc2 = (bytes[1] >> 4) & 0x0F;
        let mcc3 = bytes[2] & 0x0F;
        let mnc3 = (bytes[2] >> 4) & 0x0F;
        let mnc1 = bytes[3] & 0x0F;
        let mnc2 = (bytes[3] >> 4) & 0x0F;
        
        let mcc = format!("{}{}{}", mcc1, mcc2, mcc3);
        let mnc = if mnc3 == 0x0F {
            format!("{}{}", mnc1, mnc2)
        } else {
            format!("{}{}{}", mnc1, mnc2, mnc3)
        };
        
        // Decode TACs based on type
        let mut tacs = Vec::new();
        let tac_start = 4; // TACs start after MCC/MNC
        
        match list_type {
            0b00 => {
                // Type 00: single TAC (3 bytes)
                if bytes.len() >= tac_start + 3 {
                    let tac = format!("{:02X}{:02X}{:02X}", 
                        bytes[tac_start], bytes[tac_start + 1], bytes[tac_start + 2]);
                    tacs.push(tac);
                }
            }
            0b01 => {
                // Type 01: list of TACs (3 bytes each)
                let mut i = tac_start;
                while i + 3 <= bytes.len() {
                    let tac = format!("{:02X}{:02X}{:02X}", 
                        bytes[i], bytes[i + 1], bytes[i + 2]);
                    tacs.push(tac);
                    i += 3;
                }
            }
            0b11 => {
                // Type 11: 0 TACs, just MCC/MNC
            }
            _ => {}
        }
        
        TaiInfo {
            mcc: Some(mcc),
            mnc: Some(mnc),
            tacs,
            raw_data: None,
        }
    }
    
    /// Parse NAS message text to extract identity fields
    fn parse_nas_identity(&mut self, text: &[String]) {
        // Check message type to determine what to parse
        let mut is_registration_accept = false;
        let mut is_pdu_session_accept = false;
        let mut is_attach_accept = false;
        
        for line in text {
            if line.contains("Message type") && line.contains("Registration accept") {
                is_registration_accept = true;
                break;
            } else if line.contains("Message type") && line.contains("PDU session establishment accept") {
                is_pdu_session_accept = true;
                break;
            } else if line.contains("Message type") && line.contains("Attach accept") {
                is_attach_accept = true;
                break;
            }
        }
        
        if is_registration_accept {
            self.parse_registration_accept(text);
        } else if is_pdu_session_accept {
            self.parse_pdu_session_accept(text);
        } else if is_attach_accept {
            self.parse_attach_accept(text);
        }
    }
    
    /// Parse Registration accept message for 5G-GUTI and TAI list
    fn parse_registration_accept(&mut self, text: &[String]) {
        let mut in_guti = false;
        let mut in_tai = false;
        
        for line in text {
            let trimmed = line.trim();
            
            // 5G-GUTI section
            if trimmed.starts_with("5G-GUTI:") {
                in_guti = true;
                in_tai = false;
                continue;
            }
            
            // TAI list section
            if trimmed.starts_with("TAI list:") {
                in_tai = true;
                in_guti = false;
                continue;
            }
            
            // Reset section flags when we hit a new top-level section
            if !trimmed.is_empty() && !trimmed.starts_with(' ') && trimmed.contains(':') 
                && !trimmed.starts_with("5G-GUTI:") && !trimmed.starts_with("TAI list:") {
                in_guti = false;
                in_tai = false;
            }
            
            // Parse 5G-GUTI fields
            if in_guti {
                if let Some(value) = Self::extract_field(trimmed, "MCC =") {
                    self.nr_guti_mcc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "MNC =") {
                    self.nr_guti_mnc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Region ID =") {
                    self.nr_guti_amf_region_id = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Set ID =") {
                    self.nr_guti_amf_set_id = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Pointer =") {
                    self.nr_guti_amf_pointer = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "5G-TMSI =") {
                    self.nr_guti_5g_tmsi = Some(value);
                }
            }
            
            // Parse TAI list
            if in_tai {
                if let Some(value) = Self::extract_field(trimmed, "Data =") {
                    self.nr_tai_info = Self::decode_tai_list(&value);
                }
            }
        }
    }
    
    /// Parse PDU session establishment accept message
    fn parse_pdu_session_accept(&mut self, text: &[String]) {
        let mut in_pdu_address = false;
        let mut in_qos_rules = false;
        let mut in_qos_flow = false;
        let mut in_session_ambr = false;
        let mut in_epco = false;
        let mut saw_dns_protocol = false;
        let mut current_qos_rule: Option<QosRule> = None;
        
        // Clear QoS rules for this message
        self.nr_qos_rules.clear();
        
        for line in text {
            let trimmed = line.trim();
            
            // Section detection
            if trimmed.starts_with("PDU address:") {
                in_pdu_address = true;
                in_qos_rules = false;
                in_qos_flow = false;
                in_session_ambr = false;
                in_epco = false;
                continue;
            }
            if trimmed.starts_with("Authorized QoS rules:") {
                in_qos_rules = true;
                in_pdu_address = false;
                in_qos_flow = false;
                in_session_ambr = false;
                in_epco = false;
                continue;
            }
            if trimmed.starts_with("Authorized QoS flow descriptions:") {
                in_qos_flow = true;
                in_qos_rules = false;
                in_pdu_address = false;
                in_session_ambr = false;
                in_epco = false;
                // Save any pending QoS rule
                if let Some(rule) = current_qos_rule.take() {
                    self.nr_qos_rules.push(rule);
                }
                continue;
            }
            if trimmed.starts_with("Session AMBR:") {
                in_session_ambr = true;
                in_qos_rules = false;
                in_pdu_address = false;
                in_qos_flow = false;
                in_epco = false;
                // Save any pending QoS rule
                if let Some(rule) = current_qos_rule.take() {
                    self.nr_qos_rules.push(rule);
                }
                continue;
            }
            if trimmed.starts_with("Extended protocol configuration options:") {
                in_epco = true;
                in_qos_rules = false;
                in_pdu_address = false;
                in_qos_flow = false;
                in_session_ambr = false;
                continue;
            }
            
            // Reset section flags for other top-level sections
            if !trimmed.is_empty() && !trimmed.starts_with(' ') && trimmed.contains(':') {
                if !trimmed.starts_with("PDU address:") 
                    && !trimmed.starts_with("Authorized QoS rules:")
                    && !trimmed.starts_with("Authorized QoS flow descriptions:")
                    && !trimmed.starts_with("Session AMBR:")
                    && !trimmed.starts_with("Extended protocol configuration options:")
                    && !trimmed.starts_with("QoS rule")
                    && !trimmed.starts_with("QoS flow") {
                    in_pdu_address = false;
                    in_qos_rules = false;
                    in_qos_flow = false;
                    in_session_ambr = false;
                    in_epco = false;
                    // Save any pending QoS rule
                    if let Some(rule) = current_qos_rule.take() {
                        self.nr_qos_rules.push(rule);
                    }
                }
            }
            
            // Parse PDU address fields
            if in_pdu_address {
                if let Some(value) = Self::extract_field(trimmed, "IPv4 =") {
                    self.nr_pdu_ipv4 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "IPv6 =") {
                    self.nr_pdu_ipv6 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "PDU session type =") {
                    self.nr_pdu_session_type = Some(value);
                }
            }
            
            // Parse QoS rules
            if in_qos_rules {
                // New QoS rule starts
                if trimmed.starts_with("QoS rule") && trimmed.contains(':') {
                    // Save previous rule if any
                    if let Some(rule) = current_qos_rule.take() {
                        self.nr_qos_rules.push(rule);
                    }
                    current_qos_rule = Some(QosRule::default());
                }
                
                if let Some(ref mut rule) = current_qos_rule {
                    if let Some(value) = Self::extract_field(trimmed, "QoS rule identifier =") {
                        rule.identifier = Some(value);
                    } else if let Some(value) = Self::extract_field(trimmed, "DQR =") {
                        rule.dqr = Some(value);
                    } else if let Some(value) = Self::extract_field(trimmed, "QoS rule precedence =") {
                        rule.precedence = Some(value);
                    } else if let Some(value) = Self::extract_field(trimmed, "QFI =") {
                        rule.qfi = Some(value);
                    }
                }
            }
            
            // Parse QoS flow descriptions for 5QI
            if in_qos_flow {
                if let Some(value) = Self::extract_field(trimmed, "5QI =") {
                    self.nr_fiveqi = Some(value);
                }
            }
            
            // Parse Session AMBR
            if in_session_ambr {
                if let Some(value) = Self::extract_field(trimmed, "Session-AMBR for downlink =") {
                    self.nr_session_ambr_dl = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "Session-AMBR for uplink =") {
                    self.nr_session_ambr_ul = Some(value);
                }
            }
            
            // Parse Extended Protocol Configuration Options for DNS
            if in_epco {
                // Look for DNS Server IPv4 Address protocol
                if trimmed.contains("DNS Server IPv4 Address") {
                    saw_dns_protocol = true;
                    continue;
                }
                // Check if previous line was DNS and this is the data
                if saw_dns_protocol {
                    if let Some(value) = Self::extract_field(trimmed, "Data =") {
                        // Only set if it looks like an IP address
                        if value.contains('.') && !value.is_empty() {
                            self.nr_dns_ipv4 = Some(value);
                        }
                    }
                    saw_dns_protocol = false;
                }
                // Reset flag if we see a new Protocol ID
                if trimmed.starts_with("Protocol ID =") {
                    saw_dns_protocol = false;
                }
            }
            
            // Parse DNN (can appear anywhere)
            if let Some(value) = Self::extract_field(trimmed, "DNN =") {
                // Remove quotes if present
                self.nr_dnn = Some(value.trim_matches('"').to_string());
            }
        }
        
        // Save any remaining QoS rule
        if let Some(rule) = current_qos_rule.take() {
            self.nr_qos_rules.push(rule);
        }
    }
    
    /// Extract field value from a line like "FieldName = value"
    fn extract_field(line: &str, field_name: &str) -> Option<String> {
        if let Some(pos) = line.find(field_name) {
            let value_start = pos + field_name.len();
            let value = line[value_start..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }
    
    /// Parse 4G LTE Attach accept message
    fn parse_attach_accept(&mut self, text: &[String]) {
        let mut in_guti = false;
        let mut in_tai = false;
        let mut in_pdn_address = false;
        let mut in_pco = false;
        let mut saw_dns_ipv4_protocol = false;
        let mut saw_dns_ipv6_protocol = false;
        
        for line in text {
            let trimmed = line.trim();
            
            // Section detection
            if trimmed.starts_with("GUTI:") {
                in_guti = true;
                in_tai = false;
                in_pdn_address = false;
                in_pco = false;
                continue;
            }
            if trimmed.starts_with("TAI list:") {
                in_tai = true;
                in_guti = false;
                in_pdn_address = false;
                in_pco = false;
                continue;
            }
            if trimmed.starts_with("PDN address:") {
                in_pdn_address = true;
                in_guti = false;
                in_tai = false;
                in_pco = false;
                continue;
            }
            if trimmed.starts_with("Protocol configuration options:") {
                in_pco = true;
                in_guti = false;
                in_tai = false;
                in_pdn_address = false;
                continue;
            }
            
            // Reset section flags for other top-level sections
            // Check the original line (not trimmed) to see if it's a new top-level section
            if !line.is_empty() && !line.starts_with(' ') && line.contains(':')
                && !line.trim().starts_with("GUTI:") 
                && !line.trim().starts_with("TAI list:")
                && !line.trim().starts_with("PDN address:")
                && !line.trim().starts_with("Protocol configuration options:")
                && !line.trim().starts_with("Protocol ID") {
                in_guti = false;
                in_tai = false;
                in_pdn_address = false;
                in_pco = false;
            }
            
            // Parse 4G GUTI fields
            if in_guti {
                if let Some(value) = Self::extract_field(trimmed, "MCC =") {
                    self.lte_guti_mcc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "MNC =") {
                    self.lte_guti_mnc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "MME Group ID =") {
                    self.lte_mme_group_id = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "MME Code =") {
                    self.lte_mme_code = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "M-TMSI =") {
                    self.lte_m_tmsi = Some(value);
                }
            }
            
            // Parse TAI list
            if in_tai {
                if let Some(value) = Self::extract_field(trimmed, "Data =") {
                    self.lte_tai_info = Self::decode_tai_list(&value);
                }
            }
            
            // Parse PDN address
            if in_pdn_address {
                if let Some(value) = Self::extract_field(trimmed, "IPv4 =") {
                    self.lte_pdn_ipv4 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "IPv6 =") {
                    self.lte_pdn_ipv6 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "PDN type =") {
                    self.lte_pdn_type = Some(value);
                }
            }
            
            // Parse Protocol Configuration Options for DNS
            if in_pco {
                // Look for DNS Server IPv4 Address protocol
                if trimmed.contains("DNS Server IPv4 Address") {
                    saw_dns_ipv4_protocol = true;
                    saw_dns_ipv6_protocol = false;
                    continue;
                }
                // Look for DNS Server IPv6 Address protocol
                if trimmed.contains("DNS Server IPv6 Address") {
                    saw_dns_ipv6_protocol = true;
                    saw_dns_ipv4_protocol = false;
                    continue;
                }
                
                // Check if previous line was DNS IPv4 and this is the data
                if saw_dns_ipv4_protocol && trimmed.starts_with("Data =") {
                    if let Some(value) = Self::extract_field(trimmed, "Data =") {
                        if !value.is_empty() {
                            self.lte_dns_ipv4 = Some(value);
                            saw_dns_ipv4_protocol = false;
                        }
                    }
                }
                // Check if previous line was DNS IPv6 and this is the data
                else if saw_dns_ipv6_protocol && trimmed.starts_with("Data =") {
                    if let Some(value) = Self::extract_field(trimmed, "Data =") {
                        if !value.is_empty() {
                            self.lte_dns_ipv6 = Some(value);
                            saw_dns_ipv6_protocol = false;
                        }
                    }
                }
                
                // Reset flags if we see a new Protocol ID that's not DNS
                if trimmed.starts_with("Protocol ID =") && !trimmed.contains("DNS") {
                    saw_dns_ipv4_protocol = false;
                    saw_dns_ipv6_protocol = false;
                }
            }
            
            // Parse Access point name (can appear anywhere in ESM message container)
            if let Some(value) = Self::extract_field(trimmed, "Access point name =") {
                self.lte_apn = Some(value.trim_matches('"').to_string());
            }
        }
    }
    
    /// Render a field row with theme-aware colors
    fn render_field(ui: &mut egui::Ui, label: &str, value: &Option<String>) {
        let theme = ThemeColors::get(ui);
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(theme.text_strong).strong());
            ui.label(RichText::new(value.as_ref().map(|s| s.as_str()).unwrap_or("N/A"))
                .color(theme.text));
        });
    }
    
    /// Render a field row with a specific value (not Option)
    fn render_field_value(ui: &mut egui::Ui, label: &str, value: &str) {
        let theme = ThemeColors::get(ui);
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(theme.text_strong).strong());
            ui.label(RichText::new(value).color(theme.text));
        });
    }
}

impl EventSubscriber for Identity {
    fn name(&self) -> &'static str {
        "Identity"
    }
    
    fn on_event_added(&mut self, event: &Trace, _index: usize, context: &EventContext) {
        // Update metadata when events are added
        self.metadata = context.metadata.clone();
        
        // Parse NAS messages for identity information
        if event.layer == Layer::NAS {
            if let Some(text) = &event.text {
                self.parse_nas_identity(text);
            }
        }
    }
    
    fn on_event_focused(&mut self, event: &Trace, index: usize, context: &EventContext) {
        self.current_index = index;
        self.metadata = context.metadata.clone();
        
        // Parse the focused NAS message to update state
        // Values are preserved when navigating to other layers (RRC, etc.)
        if event.layer == Layer::NAS {
            if let Some(text) = &event.text {
                self.parse_nas_identity(text);
            }
        }
        // Note: We don't clear fields when navigating away from NAS messages
        // The values persist as state until a new NAS message updates them
    }
    
    fn on_events_cleared(&mut self) {
        log::debug!("Identity: Clearing all state");
        self.current_index = 0;
        // 4G fields
        self.lte_guti_mcc = None;
        self.lte_guti_mnc = None;
        self.lte_mme_group_id = None;
        self.lte_mme_code = None;
        self.lte_m_tmsi = None;
        self.lte_pdn_ipv4 = None;
        self.lte_pdn_ipv6 = None;
        self.lte_pdn_type = None;
        self.lte_apn = None;
        self.lte_tai_info = TaiInfo::default();
        self.lte_dns_ipv4 = None;
        self.lte_dns_ipv6 = None;
        // 5G fields
        self.nr_pdu_ipv4 = None;
        self.nr_pdu_ipv6 = None;
        self.nr_pdu_session_type = None;
        self.nr_dnn = None;
        self.nr_guti_mcc = None;
        self.nr_guti_mnc = None;
        self.nr_guti_amf_region_id = None;
        self.nr_guti_amf_set_id = None;
        self.nr_guti_amf_pointer = None;
        self.nr_guti_5g_tmsi = None;
        self.nr_qos_rules.clear();
        self.nr_fiveqi = None;
        self.nr_dns_ipv4 = None;
        self.nr_tai_info = TaiInfo::default();
        self.nr_session_ambr_dl = None;
        self.nr_session_ambr_ul = None;
    }
    
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), tramex_tools::errors::TramexError> {
        egui::Window::new("Identity")
            .open(open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        Ok(())
    }
}

impl PanelView for Identity {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.heading("NAS Identity Information");
                ui.separator();
                
                let theme = ThemeColors::get(ui);
                
                // Use Technology from metadata to determine which section to show
                match self.metadata.technology {
                    Technology::LTE => {                        
                        // PDN Address section (4G)
                        ui.group(|ui| {
                            ui.label(RichText::new("PDN Address").strong().color(theme.header));
                            Self::render_field(ui, "PDN Type:", &self.lte_pdn_type);
                            Self::render_field(ui, "IPv4:", &self.lte_pdn_ipv4);
                            Self::render_field(ui, "IPv6:", &self.lte_pdn_ipv6);
                        });
                        
                        ui.add_space(5.0);
                        
                        // Network section (4G)
                        ui.group(|ui| {
                            ui.label(RichText::new("Network").strong().color(theme.header));
                            Self::render_field(ui, "APN:", &self.lte_apn);
                            Self::render_field(ui, "DNS IPv4:", &self.lte_dns_ipv4);
                            Self::render_field(ui, "DNS IPv6:", &self.lte_dns_ipv6);
                        });
                        
                        ui.add_space(5.0);
                        
                        // GUTI section (4G - MME)
                        ui.group(|ui| {
                            ui.label(RichText::new("GUTI (MME)").strong().color(theme.header));
                            Self::render_field(ui, "MCC:", &self.lte_guti_mcc);
                            Self::render_field(ui, "MNC:", &self.lte_guti_mnc);
                            Self::render_field(ui, "MME Group ID:", &self.lte_mme_group_id);
                            Self::render_field(ui, "MME Code:", &self.lte_mme_code);
                            Self::render_field(ui, "M-TMSI:", &self.lte_m_tmsi);
                        });
                        
                        ui.add_space(5.0);
                        
                        // TAI List section (4G)
                        ui.group(|ui| {
                            ui.label(RichText::new("TAI List").strong().color(theme.header));
                            if let Some(raw) = &self.lte_tai_info.raw_data {
                                Self::render_field_value(ui, "Raw Data:", raw);
                            } else {
                                Self::render_field(ui, "MCC:", &self.lte_tai_info.mcc);
                                Self::render_field(ui, "MNC:", &self.lte_tai_info.mnc);
                                if !self.lte_tai_info.tacs.is_empty() {
                                    let tacs_str = self.lte_tai_info.tacs.join(", ");
                                    Self::render_field_value(ui, "TAC(s):", &tacs_str);
                                } else {
                                    Self::render_field_value(ui, "TAC(s):", "N/A");
                                }
                            }
                        });
                    }
                    Technology::NR => {                        
                        // PDU Address section (5G)
                        ui.group(|ui| {
                            ui.label(RichText::new("PDU Address").strong().color(theme.header));
                            Self::render_field(ui, "Session Type:", &self.nr_pdu_session_type);
                            Self::render_field(ui, "IPv4:", &self.nr_pdu_ipv4);
                            Self::render_field(ui, "IPv6:", &self.nr_pdu_ipv6);
                        });
                        
                        ui.add_space(5.0);
                        
                        // Network section (5G)
                        ui.group(|ui| {
                            ui.label(RichText::new("Network").strong().color(theme.header));
                            Self::render_field(ui, "DNN:", &self.nr_dnn);
                            Self::render_field(ui, "DNS IPv4:", &self.nr_dns_ipv4);
                        });
                        
                        ui.add_space(5.0);
                        
                        // 5G-GUTI section
                        ui.group(|ui| {
                            ui.label(RichText::new("5G-GUTI (AMF)").strong().color(theme.header));
                            Self::render_field(ui, "MCC:", &self.nr_guti_mcc);
                            Self::render_field(ui, "MNC:", &self.nr_guti_mnc);
                            Self::render_field(ui, "AMF Region ID:", &self.nr_guti_amf_region_id);
                            Self::render_field(ui, "AMF Set ID:", &self.nr_guti_amf_set_id);
                            Self::render_field(ui, "AMF Pointer:", &self.nr_guti_amf_pointer);
                            Self::render_field(ui, "5G-TMSI:", &self.nr_guti_5g_tmsi);
                        });
                        
                        ui.add_space(5.0);
                        
                        // TAI List section (5G)
                        ui.group(|ui| {
                            ui.label(RichText::new("TAI List").strong().color(theme.header));
                            if let Some(raw) = &self.nr_tai_info.raw_data {
                                Self::render_field_value(ui, "Raw Data:", raw);
                            } else {
                                Self::render_field(ui, "MCC:", &self.nr_tai_info.mcc);
                                Self::render_field(ui, "MNC:", &self.nr_tai_info.mnc);
                                if !self.nr_tai_info.tacs.is_empty() {
                                    let tacs_str = self.nr_tai_info.tacs.join(", ");
                                    Self::render_field_value(ui, "TAC(s):", &tacs_str);
                                } else {
                                    Self::render_field_value(ui, "TAC(s):", "N/A");
                                }
                            }
                        });
                        
                        ui.add_space(5.0);
                        
                        // Session AMBR section
                        ui.group(|ui| {
                            ui.label(RichText::new("Session AMBR").strong().color(theme.header));
                            Self::render_field(ui, "Downlink:", &self.nr_session_ambr_dl);
                            Self::render_field(ui, "Uplink:", &self.nr_session_ambr_ul);
                        });
                        
                        ui.add_space(5.0);
                        
                        // QoS Rules section
                        ui.group(|ui| {
                            ui.label(RichText::new("QoS Rules").strong().color(theme.header));
                            Self::render_field(ui, "5QI:", &self.nr_fiveqi);
                            
                            if self.nr_qos_rules.is_empty() {
                                Self::render_field_value(ui, "Rules:", "N/A");
                            } else {
                                for (i, rule) in self.nr_qos_rules.iter().enumerate() {
                                    ui.add_space(3.0);
                                    ui.label(RichText::new(format!("Rule {}:", i + 1))
                                        .color(theme.text_weak)
                                        .italics());
                                    Self::render_field(ui, "  Identifier:", &rule.identifier);
                                    Self::render_field(ui, "  DQR:", &rule.dqr);
                                    Self::render_field(ui, "  QoS rule precedence:", &rule.precedence);
                                    Self::render_field(ui, "  QFI:", &rule.qfi);
                                }
                            }
                        });
                    }
                    Technology::Unknown => {
                        ui.label(RichText::new("No NAS identity data available")
                            .color(theme.text_weak)
                        );
                    }
                }
            });
    }
}
