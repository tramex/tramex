//! RRC Field Viewer Panel
//! 
//! Displays parsed ASN.1 fields from RRC messages based on configurable field mappings.

use super::PanelController;
use egui::{self, Color32, RichText};
use serde_json::Value;
use tramex_tools::{
    data::{AdditionalInfos, Data, Trace},
    errors::TramexError,
    interface::layer::Layer,
};

/// Configuration for a field to extract and display
#[derive(Clone, Debug)]
pub struct FieldMapping {
    /// Display name for the field
    pub display_name: String,
    /// JSON path to the field (e.g., "message.c1.systemInformationBlockType1.cellSelectionInfo.q-RxLevMin")
    pub json_path: String,
    /// Optional unit to display (e.g., "dB", "dBm")
    pub unit: Option<String>,
}

/// Configuration for parsing a specific message type
#[derive(Clone, Debug)]
pub struct MessageConfig {
    /// Canal message name to match (e.g., "SIB1", "MIB")
    pub canal_msg: String,
    /// Fields to extract from this message type
    pub fields: Vec<FieldMapping>,
}

/// RRC Field Viewer Panel
#[derive(serde::Deserialize, serde::Serialize)]
pub struct RrcFieldViewer {
    /// Current trace index
    #[serde(skip)]
    current_index: usize,
    
    /// Parsed fields from current message
    #[serde(skip)]
    current_fields: Vec<(String, String)>, // (display_name, value)
    
    /// Current message type
    #[serde(skip)]
    current_message_type: Option<String>,
    
    /// Message configurations (not serialized, will be initialized)
    #[serde(skip)]
    message_configs: Vec<MessageConfig>,
}

impl Default for RrcFieldViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl RrcFieldViewer {
    /// Create a new RRC Field Viewer
    pub fn new() -> Self {
        let mut viewer = Self {
            current_index: 0,
            current_fields: Vec::new(),
            current_message_type: None,
            message_configs: Vec::new(),
        };
        
        // Initialize default configurations
        viewer.init_default_configs();
        viewer
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
            ],
        });
    }
    
    /// Add a new message configuration
    pub fn add_message_config(&mut self, config: MessageConfig) {
        self.message_configs.push(config);
    }
    
    /// Update displayed fields based on current trace
    fn update_fields(&mut self, trace: &Trace) {
        self.current_fields.clear();
        self.current_message_type = None;
        
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
        
        if let Some(config) = config {
            self.current_message_type = Some(canal_msg.clone());
            
            // Parse ASN.1 to JSON
            if let Some(json) = trace.parse_asn1_to_json() {                
                // Extract each configured field
                for field_mapping in &config.fields {
                    if let Some(value) = self.extract_field(&json, &field_mapping.json_path) {
                        let display_value = if let Some(unit) = &field_mapping.unit {
                            format!("{} {}", value, unit)
                        } else {
                            value
                        };
                        self.current_fields.push((
                            field_mapping.display_name.clone(),
                            display_value,
                        ));
                    }
                }
            }
        }
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
            Value::String(s) => s.clone(),
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
    
    /// UI for the panel
    fn ui(&mut self, ui: &mut egui::Ui) {
        if let Some(msg_type) = &self.current_message_type {
            ui.heading(format!("Message Type: {}", msg_type));
            ui.separator();
            
            if self.current_fields.is_empty() {
                ui.label(RichText::new("No fields configured for this message type")
                    .color(Color32::GRAY));
            } else {
                // Display fields in a table-like format
                egui::Grid::new("rrc_fields_grid")
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        for (name, value) in &self.current_fields {
                            ui.label(RichText::new(name)
                                .color(Color32::BLACK)
                                .strong());
                            ui.label(RichText::new(value)
                                .color(Color32::DARK_GRAY));
                            ui.end_row();
                        }
                    });
            }
        } else {
            ui.label(RichText::new("No RRC message selected or message type not configured")
                .color(Color32::GRAY));
            // ui.add_space(10.0);
            
            // // Show configured message types
            // ui.label(RichText::new("Configured message types:")
            //     .color(Color32::from_rgb(200, 150, 255)));
            // for config in &self.message_configs {
            //     ui.label(format!("  • {} ({} fields)", config.canal_msg, config.fields.len()));
            // }
        }
    }
}

impl PanelController for RrcFieldViewer {
    fn name(&self) -> &'static str {
        "RRC Fields"
    }
    
    fn window_title(&self) -> &'static str {
        "RRC Fields"
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data: &mut Data,
    ) -> Result<(), TramexError> {
        // Update fields if trace changed
        if data.is_different_index(self.current_index) {
            self.current_index = data.current_index;
            if let Some(trace) = data.get_current_trace() {
                self.update_fields(trace);
            }
        }
        
        egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        
        Ok(())
    }

    fn clear(&mut self) {
        self.current_index = 0;
        self.current_fields.clear();
        self.current_message_type = None;
    }
}
