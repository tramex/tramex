//! RRC Field Viewer Panel
//! 
//! Displays parsed ASN.1 fields from RRC messages based on configurable field mappings.

use crate::event_system::{EventSubscriber, EventContext};
use egui::{self, Color32, RichText};
use serde_json::Value;
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::{layer::Layer, parse_config::FileMetadata},
};
use crate::panels::PanelView;

/// Configuration for a field to extract and display
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FieldMapping {
    /// Display name for the field
    pub display_name: String,
    /// JSON path to the field (e.g., "message.c1.systemInformationBlockType1.cellSelectionInfo.q-RxLevMin")
    pub json_path: String,
    /// Optional unit to display (e.g., "dB", "dBm")
    pub unit: Option<String>,
}

/// Configuration for parsing a specific message type
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MessageConfig {
    /// Canal message name to match (e.g., "SIB1", "MIB")
    pub canal_msg: String,
    /// Fields to extract from this message type
    pub fields: Vec<FieldMapping>,
}

/// RRC Field Viewer Panel
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BstConfig {
    /// Current index
    #[serde(skip)]
    current_index: usize,
    
    /// Cached metadata for UI rendering
    #[serde(skip)]
    metadata: FileMetadata,
    
    /// Parsed fields from SIB1
    #[serde(skip)]
    sib1_fields: Vec<(String, String)>,
    
    /// Parsed fields from SIB2
    #[serde(skip)]
    sib2_fields: Vec<(String, String)>,
    
    /// Parsed fields from SIB3
    #[serde(skip)]
    sib3_fields: Vec<(String, String)>,
    
    /// Message configurations (not serialized, will be initialized)
    #[serde(skip)]
    message_configs: Vec<MessageConfig>,
}

impl Default for BstConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BstConfig {
    /// Create a new RRC Field Viewer
    pub fn new() -> Self {
        Self {
            current_index: 0,
            metadata: FileMetadata::default(),
            sib1_fields: Vec::new(),
            sib2_fields: Vec::new(),
            sib3_fields: Vec::new(),
            message_configs: Vec::new(),
        }
    }
    
    /// Initialize default field configurations
    fn init_default_configs(&mut self) {
        // SIB1 Configuration
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB1".to_string(),
            fields: vec![
                //4G
                FieldMapping {
                    display_name: "mcc".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityList[0].plmn-Identity.mcc".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "mnc".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityList[0].plmn-Identity.mnc".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "cellId".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.cellIdentity".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "trackingAreaCode".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.trackingAreaCode".to_string(),
                    unit: None,
                },
                
                // 5G
                FieldMapping {
                    display_name: "q-RxLevMin".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellSelectionInfo.q-RxLevMin".to_string(),
                    unit: Some("dB".to_string()),
                },
                FieldMapping {
                    display_name: "q-QualMin".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellSelectionInfo.q-QualMin".to_string(),
                    unit: Some("dB".to_string()),
                },
                FieldMapping {
                    display_name: "mcc".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityInfoList[0].plmn-IdentityList[0].mcc".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "mnc".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityInfoList[0].plmn-IdentityList[0].mnc".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "cellId".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityInfoList[0].cellIdentity".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "trackingAreaCode".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityInfoList[0].trackingAreaCode".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "DL frequencyBandIndicatorNR".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.servingCellConfigCommon.downlinkConfigCommon.frequencyInfoDL.frequencyBandList[0].freqBandIndicatorNR".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "DL subcarrierSpacing".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.servingCellConfigCommon.downlinkConfigCommon.frequencyInfoDL.scs-SpecificCarrierList[0].subcarrierSpacing".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "DL carrierBandwidth".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.servingCellConfigCommon.downlinkConfigCommon.frequencyInfoDL.scs-SpecificCarrierList[0].carrierBandwidth".to_string(),
                    unit: Some("RB".to_string()),
                },
                FieldMapping {
                    display_name: "UL subcarrierSpacing".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.servingCellConfigCommon.uplinkConfigCommon.frequencyInfoUL.scs-SpecificCarrierList[0].subcarrierSpacing".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "UL carrierBandwidth".to_string(),
                    json_path: "message.c1.systemInformationBlockType1.servingCellConfigCommon.uplinkConfigCommon.frequencyInfoUL.scs-SpecificCarrierList[0].carrierBandwidth".to_string(),
                    unit: Some("RB".to_string()),
                },
            ],
        });
        
        // SIB2 Configuration
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB2".to_string(),
            fields: vec![
                FieldMapping {
                    display_name: "q-Hyst".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib2.cellReselectionInfoCommon.q-Hyst".to_string(),
                    unit: Some("dB".to_string()),
                },
                FieldMapping {
                    display_name: "q-RxLevelMin".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib2.intraFreqCellReselectionInfo.q-RxLevelMin".to_string(),
                    unit: Some("dB".to_string()),
                },
                FieldMapping {
                    display_name: "s-IntraSearchP".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib2.intraFreqCellReselectionInfo.s-IntraSearchP".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "t-ReselectionNR".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib2.intraFreqCellReselectionInfo.q-RxLevelMin".to_string(),
                    unit: None,
                },
                // Add more SIB2 fields as needed
            ],
        });
        
        // SIB3 Configuration
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB3".to_string(),
            fields: vec![
                FieldMapping {
                    display_name: "intraFreqNeighCellList".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib3.intraFreqNeighCellList".to_string(),
                    unit: None,
                },
                // Add more SIB3 fields as needed
            ],
        });
    }
    
    /// Add a new message configuration
    pub fn add_message_config(&mut self, config: MessageConfig) {
        self.message_configs.push(config);
    }
    
    /// Update displayed fields based on current trace
    /// Only updates when a matching message type is found (persists previous values otherwise)
    fn update_fields(&mut self, trace: &Trace) {
        // Only process RRC messages
        if !matches!(trace.layer, Layer::RRC) {
            return;
        }
        
        // Get canal_msg from additional_infos
        let canal_msg = match &trace.additional_infos {
            AdditionalInfos::RRCInfos(infos) => &infos.canal_msg,
            _ => return,
        };
        
        // Find matching configuration
        let config = self.message_configs.iter()
            .find(|c| c.canal_msg.eq_ignore_ascii_case(canal_msg));
        
        // Only update if we found a matching configuration
        if let Some(config) = config {
            // Parse ASN.1 to JSON
            if let Some(json) = trace.parse_asn1_to_json() {
                let mut fields = Vec::new();
                
                // Extract each configured field
                for field_mapping in &config.fields {
                    if let Some(value) = self.extract_field(&json, &field_mapping.json_path) {
                        let display_value = if let Some(unit) = &field_mapping.unit {
                            format!("{} {}", value, unit)
                        } else {
                            value
                        };
                        fields.push((
                            field_mapping.display_name.clone(),
                            display_value,
                        ));
                    }
                }
                
                // Store in the appropriate section based on message type
                match canal_msg.to_uppercase().as_str() {
                    "SIB1" => self.sib1_fields = fields,
                    "SIB2" => self.sib2_fields = fields,
                    "SIB3" => self.sib3_fields = fields,
                    _ => {} // Unknown SIB type, ignore
                }
            }
        }
        // If no matching config found, keep the previous fields displayed
    }
    
    /// Extract a field from JSON using a dot-separated path
    /// Supports array indexing with [N] syntax in the path
    fn extract_field(&self, json: &Value, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;
        
        for part in parts {
            // Check if this part has array indexing like "field[0]"
            if let Some(bracket_pos) = part.find('[') {
                let field_name = &part[..bracket_pos];
                let index_str = &part[bracket_pos+1..part.len()-1];
                
                // Get the field
                current = current.get(field_name)?;
                
                // Index into the array
                if let Value::Array(arr) = current {
                    let index: usize = index_str.parse().ok()?;
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            } else {
                current = current.get(part)?;
            }
        }
        
        // Convert value to string
        Some(match current {
            Value::String(s) => {
                // Handle ASN.1 enum values like "dB3" -> "3"
                // This allows the unit to be added separately as "3 dB"
                if s.starts_with("dB") && s.len() > 2 {
                    s[2..].to_string()
                } else {
                    s.clone()
                }
            },
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Object(obj) => {
                // For complex objects, try to extract meaningful info
                if let Some(decimal) = obj.get("decimal") {
                    decimal.to_string()
                } else if let Some(hex) = obj.get("hex") {
                    format!("0x{}", hex.as_str().unwrap_or("?"))
                } else {
                    serde_json::to_string(obj).unwrap_or_else(|_| "?".to_string())
                }
            }
            Value::Array(arr) => {
                // Display array contents
                let items: Vec<String> = arr.iter()
                    .map(|v| match v {
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        _ => "?".to_string(),
                    })
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Null => "null".to_string(),
        })
    }
    
    /// UI for the panel with metadata
    fn ui_with_metadata(&mut self, ui: &mut egui::Ui, metadata: &FileMetadata) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // === GENERAL SECTION ===
                ui.heading("General");
                ui.separator();
                
                egui::Grid::new("bst_general_grid")
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        // Technology
                        ui.label(RichText::new("Technology")
                            .color(Color32::BLACK)
                            .strong());
                        ui.label(RichText::new(format!("{}", metadata.technology))
                            .color(Color32::DARK_GRAY));
                        ui.end_row();
                        
                        // PCI
                        if let Some(pci) = metadata.pci {
                            ui.label(RichText::new("PCI")
                                .color(Color32::BLACK)
                                .strong());
                            ui.label(RichText::new(format!("{}", pci))
                                .color(Color32::DARK_GRAY));
                            ui.end_row();
                        }
                        
                        // Mode
                        if let Some(ref mode) = metadata.mode {
                            ui.label(RichText::new("Mode")
                                .color(Color32::BLACK)
                                .strong());
                            ui.label(RichText::new(mode)
                                .color(Color32::DARK_GRAY));
                            ui.end_row();
                        }
                        
                        // ARFCN
                        if let Some(arfcn) = metadata.arfcn {
                            ui.label(RichText::new("ARFCN")
                                .color(Color32::BLACK)
                                .strong());
                            ui.label(RichText::new(format!("{}", arfcn))
                                .color(Color32::DARK_GRAY));
                            ui.end_row();
                        }
                        
                        // IO Mode (MIMO/SISO)
                        if let Some(ref io_mode) = metadata.io_mode {
                            ui.label(RichText::new("I/O Mode")
                                .color(Color32::BLACK)
                                .strong());
                            ui.label(RichText::new(io_mode)
                                .color(Color32::DARK_GRAY));
                            ui.end_row();
                        }
                    });
                
                // === SIB1 SECTION ===
                if !self.sib1_fields.is_empty() {
                    ui.add_space(15.0);
                    ui.heading("SIB1");
                    ui.separator();
                    
                    egui::Grid::new("bst_sib1_grid")
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (name, value) in &self.sib1_fields {
                                ui.label(RichText::new(name)
                                    .color(Color32::BLACK)
                                    .strong());
                                ui.label(RichText::new(value)
                                    .color(Color32::DARK_GRAY));
                                ui.end_row();
                            }
                        });
                }
                
                // === SIB2 SECTION ===
                if !self.sib2_fields.is_empty() {
                    ui.add_space(15.0);
                    ui.heading("SIB2");
                    ui.separator();
                    
                    egui::Grid::new("bst_sib2_grid")
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (name, value) in &self.sib2_fields {
                                ui.label(RichText::new(name)
                                    .color(Color32::BLACK)
                                    .strong());
                                ui.label(RichText::new(value)
                                    .color(Color32::DARK_GRAY));
                                ui.end_row();
                            }
                        });
                }
                
                // === SIB3 SECTION ===
                if !self.sib3_fields.is_empty() {
                    ui.add_space(15.0);
                    ui.heading("SIB3");
                    ui.separator();
                    
                    egui::Grid::new("bst_sib3_grid")
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (name, value) in &self.sib3_fields {
                                ui.label(RichText::new(name)
                                    .color(Color32::BLACK)
                                    .strong());
                                ui.label(RichText::new(value)
                                    .color(Color32::DARK_GRAY));
                                ui.end_row();
                            }
                        });
                }
            });
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for BstConfig {
    fn on_event_added(&mut self, _event: &Trace, _index: usize, context: &EventContext) {
        // Update cached metadata
        self.metadata = context.metadata.clone();
    }
    
    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates to an event, extract fields from it
        self.current_index = index;
        self.update_fields(event);
    }
    
    fn on_events_cleared(&mut self) {
        log::debug!("BST Config: Clearing all fields");
        self.current_index = 0;
        self.sib1_fields.clear();
        self.sib2_fields.clear();
        self.sib3_fields.clear();
    }
    
    fn name(&self) -> &'static str {
        "BST Config"
    }
    
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        // Clone metadata to avoid borrow checker issues
        let metadata = self.metadata.clone();
        egui::Window::new("BST Config")
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .open(open)
            .show(ctx, |ui| {
                self.ui_with_metadata(ui, &metadata);
            });
        Ok(())
    }
}


// PanelView implementation for rendering UI
impl super::PanelView for BstConfig {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Clone metadata to avoid borrow checker issues
        let metadata = self.metadata.clone();
        self.ui_with_metadata(ui, &metadata);
    }
}
