//! Identity Panel
//! 
//! Displays parsed identity information from NAS messages (PDU session establishment, etc.)

use crate::event_system::{EventSubscriber, EventContext};
use egui::{self, Color32, RichText};
use tramex_tools::{
    data::Trace,
    interface::{layer::Layer, parse_config::FileMetadata},
};
use crate::panels::PanelView;

/// Identity Panel - displays NAS identity information
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Identity {
    /// Current index
    #[serde(skip)]
    current_index: usize,
    
    /// Cached metadata for UI rendering
    #[serde(skip)]
    metadata: FileMetadata,
    
    /// PDU IPv4 address
    #[serde(skip)]
    pdu_ipv4: Option<String>,
    
    /// PDU IPv6 address
    #[serde(skip)]
    pdu_ipv6: Option<String>,
    
    /// PDU session type
    #[serde(skip)]
    pdu_session_type: Option<String>,
    
    /// DNN (Data Network Name)
    #[serde(skip)]
    dnn: Option<String>,
    
    /// 5G-GUTI MCC
    #[serde(skip)]
    guti_mcc: Option<String>,
    
    /// 5G-GUTI MNC
    #[serde(skip)]
    guti_mnc: Option<String>,
    
    /// 5G-GUTI AMF Region ID
    #[serde(skip)]
    guti_amf_region_id: Option<String>,
    
    /// 5G-GUTI AMF Set ID
    #[serde(skip)]
    guti_amf_set_id: Option<String>,
    
    /// 5G-GUTI AMF Pointer
    #[serde(skip)]
    guti_amf_pointer: Option<String>,
    
    /// 5G-GUTI 5G-TMSI
    #[serde(skip)]
    guti_5g_tmsi: Option<String>,
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
            pdu_ipv4: None,
            pdu_ipv6: None,
            pdu_session_type: None,
            dnn: None,
            guti_mcc: None,
            guti_mnc: None,
            guti_amf_region_id: None,
            guti_amf_set_id: None,
            guti_amf_pointer: None,
            guti_5g_tmsi: None,
        }
    }
    
    /// Parse NAS message text to extract identity fields
    fn parse_nas_identity(&mut self, text: &[String]) {
        // Check message type to determine what to parse
        let mut is_registration_accept = false;
        let mut is_pdu_session_accept = false;
        
        for line in text {
            if line.contains("Message type") && line.contains("Registration accept") {
                is_registration_accept = true;
                break;
            } else if line.contains("Message type") && line.contains("PDU session establishment accept") {
                is_pdu_session_accept = true;
                break;
            }
        }
        
        if is_registration_accept {
            self.parse_registration_accept(text);
        } else if is_pdu_session_accept {
            self.parse_pdu_session_accept(text);
        }
    }
    
    /// Parse Registration accept message for 5G-GUTI
    fn parse_registration_accept(&mut self, text: &[String]) {
        let mut in_guti = false;
        
        for line in text {
            let trimmed = line.trim();
            
            // 5G-GUTI section
            if trimmed.starts_with("5G-GUTI:") {
                in_guti = true;
                continue;
            }
            
            // Reset section flag when we hit a new top-level section
            if !trimmed.is_empty() && !trimmed.starts_with(' ') && trimmed.contains(':') {
                if !trimmed.starts_with("5G-GUTI:") {
                    in_guti = false;
                }
            }
            
            // Parse 5G-GUTI fields
            if in_guti {
                if let Some(value) = Self::extract_field(trimmed, "MCC =") {
                    self.guti_mcc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "MNC =") {
                    self.guti_mnc = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Region ID =") {
                    self.guti_amf_region_id = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Set ID =") {
                    self.guti_amf_set_id = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "AMF Pointer =") {
                    self.guti_amf_pointer = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "5G-TMSI =") {
                    self.guti_5g_tmsi = Some(value);
                }
            }
        }
    }
    
    /// Parse PDU session establishment accept message for PDU address and DNN
    fn parse_pdu_session_accept(&mut self, text: &[String]) {
        let mut in_pdu_address = false;
        
        for line in text {
            let trimmed = line.trim();
            
            // PDU address section
            if trimmed.starts_with("PDU address:") {
                in_pdu_address = true;
                continue;
            }
            
            // Reset section flag when we hit a new top-level section
            if !trimmed.is_empty() && !trimmed.starts_with(' ') && trimmed.contains(':') {
                if !trimmed.starts_with("PDU address:") {
                    in_pdu_address = false;
                }
            }
            
            // Parse PDU address fields
            if in_pdu_address {
                if let Some(value) = Self::extract_field(trimmed, "IPv4 =") {
                    self.pdu_ipv4 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "IPv6 =") {
                    self.pdu_ipv6 = Some(value);
                } else if let Some(value) = Self::extract_field(trimmed, "PDU session type =") {
                    self.pdu_session_type = Some(value);
                }
            }
            
            // Parse DNN (can appear anywhere)
            if let Some(value) = Self::extract_field(trimmed, "DNN =") {
                // Remove quotes if present
                self.dnn = Some(value.trim_matches('"').to_string());
            }
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
    
    /// Render a field row
    fn render_field(ui: &mut egui::Ui, label: &str, value: &Option<String>) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(Color32::LIGHT_BLUE).strong());
            ui.label(value.as_ref().map(|s| s.as_str()).unwrap_or("N/A"));
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
        self.pdu_ipv4 = None;
        self.pdu_ipv6 = None;
        self.pdu_session_type = None;
        self.dnn = None;
        self.guti_mcc = None;
        self.guti_mnc = None;
        self.guti_amf_region_id = None;
        self.guti_amf_set_id = None;
        self.guti_amf_pointer = None;
        self.guti_5g_tmsi = None;
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
        ui.heading("NAS Identity Information");
        ui.separator();
        
        // PDU Address section
        ui.group(|ui| {
            ui.label(RichText::new("PDU Address").strong().color(Color32::YELLOW));
            Self::render_field(ui, "Session Type:", &self.pdu_session_type);
            Self::render_field(ui, "IPv4:", &self.pdu_ipv4);
            Self::render_field(ui, "IPv6:", &self.pdu_ipv6);
        });
        
        ui.add_space(5.0);
        
        // Network section
        ui.group(|ui| {
            ui.label(RichText::new("Network").strong().color(Color32::YELLOW));
            Self::render_field(ui, "DNN:", &self.dnn);
        });
        
        ui.add_space(5.0);
        
        // 5G-GUTI section
        ui.group(|ui| {
            ui.label(RichText::new("5G-GUTI").strong().color(Color32::YELLOW));
            Self::render_field(ui, "MCC:", &self.guti_mcc);
            Self::render_field(ui, "MNC:", &self.guti_mnc);
            Self::render_field(ui, "AMF Region ID:", &self.guti_amf_region_id);
            Self::render_field(ui, "AMF Set ID:", &self.guti_amf_set_id);
            Self::render_field(ui, "AMF Pointer:", &self.guti_amf_pointer);
            Self::render_field(ui, "5G-TMSI:", &self.guti_5g_tmsi);
        });
    }
}
