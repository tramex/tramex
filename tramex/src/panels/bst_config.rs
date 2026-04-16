//! RRC Field Viewer Panel
//!
//! Displays parsed ASN.1 fields from RRC messages based on configurable field mappings.

use crate::event_system::{EventContext, EventSubscriber};
use crate::theme::ThemeColors;
use egui::{self, RichText};
use serde_json::Value;
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::{layer::Layer, parse_config::FileMetadata},
};

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

/// S-NSSAI (Single Network Slice Selection Assistance Information)
#[derive(Clone, Debug, Default)]
pub struct Nssai {
    /// Slice/Service Type
    pub sst: Option<String>,
    /// Slice Differentiator
    pub sd: Option<String>,
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

    /// Allowed NSSAI list (from NAS Registration accept)
    #[serde(skip)]
    allowed_nssai: Vec<Nssai>,

    /// Configured NSSAI list (from NAS Registration accept)
    #[serde(skip)]
    configured_nssai: Vec<Nssai>,
}

impl Default for BstConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BstConfig {
    /// Create a new RRC Field Viewer
    pub fn new() -> Self {
        let mut instance = Self {
            current_index: 0,
            metadata: FileMetadata::default(),
            sib1_fields: Vec::new(),
            sib2_fields: Vec::new(),
            sib3_fields: Vec::new(),
            message_configs: Vec::new(),
            allowed_nssai: Vec::new(),
            configured_nssai: Vec::new(),
        };
        instance.init_default_configs();
        instance
    }

    /// Convert hex string (0x01) to decimal
    fn hex_to_decimal(hex_str: &str) -> Option<u32> {
        let hex_str = hex_str.trim();
        if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
            u32::from_str_radix(&hex_str[2..], 16).ok()
        } else {
            hex_str.parse().ok()
        }
    }

    /// Get SST description
    fn sst_description(sst: u32) -> &'static str {
        match sst {
            1 => "eMBB",
            2 => "URLLC",
            3 => "mMTC",
            4 => "V2X",
            _ => "Unknown",
        }
    }

    /// Parse NAS message for NSSAI information
    fn parse_nas_nssai(&mut self, text: &[String]) {
        // Check if this is a Registration accept message
        let is_registration_accept = text
            .iter()
            .any(|line| line.contains("Message type") && line.contains("Registration accept"));

        if !is_registration_accept {
            return;
        }

        let mut in_allowed_nssai = false;
        let mut in_configured_nssai = false;
        let mut in_snssai = false;
        let mut current_nssai = Nssai::default();
        let mut current_section_is_allowed = false;

        // Clear previous values
        self.allowed_nssai.clear();
        self.configured_nssai.clear();

        for line in text {
            let trimmed = line.trim();

            // Section detection - save previous NSSAI before switching sections
            if trimmed.starts_with("Allowed NSSAI:") {
                // Save any pending NSSAI from previous section
                if in_snssai && current_nssai.sst.is_some() {
                    if current_section_is_allowed {
                        self.allowed_nssai.push(current_nssai.clone());
                    } else if in_configured_nssai {
                        self.configured_nssai.push(current_nssai.clone());
                    }
                }
                in_allowed_nssai = true;
                in_configured_nssai = false;
                in_snssai = false;
                current_section_is_allowed = true;
                current_nssai = Nssai::default();
                continue;
            }
            if trimmed.starts_with("Configured NSSAI:") {
                // Save any pending NSSAI from previous section
                if in_snssai && current_nssai.sst.is_some()
                    && current_section_is_allowed {
                        self.allowed_nssai.push(current_nssai.clone());
                    }
                in_configured_nssai = true;
                in_allowed_nssai = false;
                in_snssai = false;
                current_section_is_allowed = false;
                current_nssai = Nssai::default();
                continue;
            }

            // Reset when hitting other top-level sections (not indented, has colon, not NSSAI related)
            if !trimmed.is_empty()
                && !line.starts_with("        ") // Check original line indentation
                && trimmed.contains(':')
                && !trimmed.starts_with("S-NSSAI")
                && !trimmed.contains("SST")
                && !trimmed.contains("SD")
                && !trimmed.contains("Length")
            {
                // Save any pending NSSAI before leaving section
                if in_snssai && current_nssai.sst.is_some() {
                    if current_section_is_allowed {
                        self.allowed_nssai.push(current_nssai.clone());
                    } else if in_configured_nssai {
                        self.configured_nssai.push(current_nssai.clone());
                    }
                }
                in_allowed_nssai = false;
                in_configured_nssai = false;
                in_snssai = false;
                current_nssai = Nssai::default();
            }

            // Parse S-NSSAI entries
            if in_allowed_nssai || in_configured_nssai {
                if trimmed.starts_with("S-NSSAI") && !trimmed.contains("=") {
                    // Save previous NSSAI if exists
                    if in_snssai && current_nssai.sst.is_some() {
                        if current_section_is_allowed {
                            self.allowed_nssai.push(current_nssai.clone());
                        } else {
                            self.configured_nssai.push(current_nssai.clone());
                        }
                    }
                    current_nssai = Nssai::default();
                    in_snssai = true;
                    continue;
                }

                if in_snssai {
                    if let Some(pos) = trimmed.find("SST =") {
                        let value = trimmed[pos + 5..].trim();
                        // Convert hex to decimal and add description
                        if let Some(decimal) = Self::hex_to_decimal(value) {
                            let desc = Self::sst_description(decimal);
                            current_nssai.sst = Some(format!("{} ({})", decimal, desc));
                        } else {
                            current_nssai.sst = Some(value.to_string());
                        }
                    } else if let Some(pos) = trimmed.find("SD =") {
                        let value = trimmed[pos + 4..].trim();
                        // Convert hex to decimal
                        if let Some(decimal) = Self::hex_to_decimal(value) {
                            current_nssai.sd = Some(decimal.to_string());
                        } else {
                            current_nssai.sd = Some(value.to_string());
                        }
                    }
                }
            }
        }

        // Save last NSSAI if exists
        if in_snssai && current_nssai.sst.is_some() {
            if current_section_is_allowed {
                self.allowed_nssai.push(current_nssai);
            } else if in_configured_nssai {
                self.configured_nssai.push(current_nssai);
            }
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

        // SIB2 Configuration (5G NR)
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB2".to_string(),
            fields: vec![
                // 5G NR paths
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
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib2.intraFreqCellReselectionInfo.t-ReselectionNR".to_string(),
                    unit: None,
                },
            ],
        });

        // SIB3 Configuration (5G NR)
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB3".to_string(),
            fields: vec![
                FieldMapping {
                    display_name: "intraFreqNeighCellList".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation.sib-TypeAndInfo.sib3.intraFreqNeighCellList".to_string(),
                    unit: None,
                },
            ],
        });

        // SIB Configuration (4G LTE - combined SIB2 & SIB3 in same message)
        self.message_configs.push(MessageConfig {
            canal_msg: "SIB".to_string(),
            fields: vec![
                // SIB2 fields (4G)
                // SIB3 fields (4G)
                FieldMapping {
                    display_name: "q-Hyst".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation-r8.sib-TypeAndInfo.sib3.cellReselectionInfoCommon.q-Hyst".to_string(),
                    unit: Some("dB".to_string()),
                },
                FieldMapping {
                    display_name: "s-NonIntraSearch".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation-r8.sib-TypeAndInfo.sib3.cellReselectionServingFreqInfo.s-NonIntraSearch".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "q-RxLevMin".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation-r8.sib-TypeAndInfo.sib3.intraFreqCellReselectionInfo.q-RxLevMin".to_string(),
                    unit: Some("dBm".to_string()),
                },
                FieldMapping {
                    display_name: "s-IntraSearch".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation-r8.sib-TypeAndInfo.sib3.intraFreqCellReselectionInfo.s-IntraSearch".to_string(),
                    unit: None,
                },
                FieldMapping {
                    display_name: "neighCellConfig".to_string(),
                    json_path: "message.c1.systemInformation.criticalExtensions.systemInformation-r8.sib-TypeAndInfo.sib3.intraFreqCellReselectionInfo.neighCellConfig".to_string(),
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

        log::debug!("BstConfig: Processing RRC message: {}", canal_msg);

        // Find matching configuration
        let config = self
            .message_configs
            .iter()
            .find(|c| c.canal_msg.eq_ignore_ascii_case(canal_msg));

        // Only update if we found a matching configuration
        if let Some(config) = config {
            log::debug!("BstConfig: Found config for {}", canal_msg);
            // Parse ASN.1 to JSON
            if let Some(json) = trace.parse_asn1_to_json() {
                // log::debug!("BstConfig: JSON structure: {}", serde_json::to_string_pretty(&json).unwrap_or_default());
                let mut fields = Vec::new();

                // Extract each configured field
                for field_mapping in &config.fields {
                    if let Some(value) = self.extract_field(&json, &field_mapping.json_path) {
                        let display_value = if let Some(unit) = &field_mapping.unit {
                            format!("{} {}", value, unit)
                        } else {
                            value
                        };
                        fields.push((field_mapping.display_name.clone(), display_value));
                    } else {
                        log::debug!(
                            "BstConfig: Field not found: {} at path {}",
                            field_mapping.display_name,
                            field_mapping.json_path
                        );
                    }
                }

                // Store in the appropriate section based on message type
                match canal_msg.to_uppercase().as_str() {
                    "SIB1" => {
                        log::debug!("BstConfig: Updating SIB1 with {} fields", fields.len());
                        self.sib1_fields = fields;
                    }
                    "SIB2" => {
                        log::debug!("BstConfig: Updating SIB2 with {} fields", fields.len());
                        self.sib2_fields = fields;
                    }
                    "SIB3" => {
                        log::debug!("BstConfig: Updating SIB3 with {} fields", fields.len());
                        self.sib3_fields = fields;
                    }
                    "SIB" => {
                        // 4G LTE combined SIB message - split fields into SIB2 and SIB3
                        log::debug!("BstConfig: Updating SIB with {} fields", fields.len());
                        // Fields are ordered: SIB2 fields first, then SIB3 fields
                        // SIB3 fields start with q-Hyst (from cellReselectionInfoCommon)
                        self.sib3_fields = fields;
                    }
                    _ => {} // Unknown SIB type, ignore
                }
            } else {
                log::warn!("BstConfig: Failed to parse ASN.1 to JSON for {}", canal_msg);
            }
        } else {
            log::debug!("BstConfig: No config found for message: {}", canal_msg);
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
                let index_str = &part[bracket_pos + 1..part.len() - 1];

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
            }
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
                // Check if it's a simple array of numbers (like MCC/MNC)
                let is_simple_array = arr.iter().all(|v| v.is_number() || v.is_string());
                if is_simple_array && arr.len() <= 10 {
                    // Compact format: [0, 0, 1]
                    let values: Vec<String> = arr
                        .iter()
                        .map(|v| match v {
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => s.clone(),
                            _ => "?".to_string(),
                        })
                        .collect();
                    format!("[{}]", values.join(", "))
                } else {
                    // Pretty format for complex arrays
                    serde_json::to_string_pretty(arr).unwrap_or_else(|_| "[]".to_string())
                }
            }
            Value::Null => "null".to_string(),
        })
    }

    /// UI for the panel with metadata
    fn ui_with_metadata(&mut self, ui: &mut egui::Ui, metadata: &FileMetadata) {
        let theme = ThemeColors::get(ui);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // === GENERAL SECTION ===
            ui.heading("General");
            ui.separator();

            egui::Grid::new("bst_general_grid").spacing([20.0, 8.0]).show(ui, |ui| {
                // Technology
                ui.label(RichText::new("Technology").color(theme.text_strong).strong());
                ui.label(RichText::new(format!("{}", metadata.technology)).color(theme.text));
                ui.end_row();

                // PCI
                if let Some(pci) = metadata.pci {
                    ui.label(RichText::new("PCI").color(theme.text_strong).strong());
                    ui.label(RichText::new(format!("{}", pci)).color(theme.text));
                    ui.end_row();
                }

                // Mode
                if let Some(ref mode) = metadata.mode {
                    ui.label(RichText::new("Mode").color(theme.text_strong).strong());
                    ui.label(RichText::new(mode).color(theme.text));
                    ui.end_row();
                }

                // ARFCN
                if let Some(arfcn) = metadata.arfcn {
                    ui.label(RichText::new("ARFCN").color(theme.text_strong).strong());
                    ui.label(RichText::new(format!("{}", arfcn)).color(theme.text));
                    ui.end_row();
                }

                // IO Mode (MIMO/SISO)
                if let Some(ref io_mode) = metadata.io_mode {
                    ui.label(RichText::new("I/O Mode").color(theme.text_strong).strong());
                    ui.label(RichText::new(io_mode).color(theme.text));
                    ui.end_row();
                }
            });

            // === SIB1 SECTION ===
            if !self.sib1_fields.is_empty() {
                ui.add_space(15.0);
                ui.heading("SIB1");
                ui.separator();

                egui::Grid::new("bst_sib1_grid").spacing([20.0, 8.0]).show(ui, |ui| {
                    for (name, value) in &self.sib1_fields {
                        ui.label(RichText::new(name).color(theme.text_strong).strong());
                        ui.label(RichText::new(value).color(theme.text));
                        ui.end_row();
                    }
                });
            }

            // === SIB2 SECTION ===
            if !self.sib2_fields.is_empty() {
                ui.add_space(15.0);
                ui.heading("SIB2");
                ui.separator();

                egui::Grid::new("bst_sib2_grid").spacing([20.0, 8.0]).show(ui, |ui| {
                    for (name, value) in &self.sib2_fields {
                        ui.label(RichText::new(name).color(theme.text_strong).strong());
                        ui.label(RichText::new(value).color(theme.text));
                        ui.end_row();
                    }
                });
            }

            // === SIB3 SECTION ===
            if !self.sib3_fields.is_empty() {
                ui.add_space(15.0);
                ui.heading("SIB3");
                ui.separator();

                // Use vertical layout for fields that may contain multi-line values
                for (name, value) in &self.sib3_fields {
                    ui.add_space(8.0);
                    ui.label(RichText::new(name).color(theme.text_strong).strong());

                    // Check if value contains newlines (multi-line structure)
                    if value.contains('\n') {
                        // Use monospace font for structured data
                        ui.label(RichText::new(value).color(theme.text).family(egui::FontFamily::Monospace));
                    } else {
                        ui.label(RichText::new(value).color(theme.text));
                    }
                }
            }

            // === NSSAI SECTION ===
            if !self.configured_nssai.is_empty() {
                ui.add_space(15.0);
                ui.heading("NSSAI");
                ui.separator();

                // Build set of allowed SST values for quick lookup
                let allowed_ssts: std::collections::HashSet<_> =
                    self.allowed_nssai.iter().filter_map(|n| n.sst.as_ref()).collect();

                egui::Grid::new("nssai_grid").spacing([20.0, 4.0]).show(ui, |ui| {
                    for nssai in &self.configured_nssai {
                        let sst = nssai.sst.as_deref().unwrap_or("N/A");
                        let sd = nssai.sd.as_deref().unwrap_or("-");

                        // Check if this SST is in allowed list
                        let is_allowed = nssai.sst.as_ref().map(|s| allowed_ssts.contains(s)).unwrap_or(false);

                        let label = if is_allowed {
                            RichText::new(format!("SST={}, SD={} (allowed)", sst, sd)).color(theme.text_strong)
                        } else {
                            RichText::new(format!("SST={}, SD={}", sst, sd)).color(theme.text)
                        };
                        ui.label(label);
                        ui.end_row();
                    }
                });
            }
        });
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for BstConfig {
    fn on_event_added(&mut self, event: &Trace, _index: usize, context: &EventContext) {
        // Update cached metadata
        self.metadata = context.metadata.clone();
        // Update RRC fields
        self.update_fields(event);
        // Parse NAS messages for NSSAI
        if event.layer == Layer::NAS
            && let Some(text) = &event.text {
                self.parse_nas_nssai(text);
            }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates to an event, extract fields from it
        self.current_index = index;
        self.update_fields(event);
        // Parse NAS messages for NSSAI
        if event.layer == Layer::NAS
            && let Some(text) = &event.text {
                self.parse_nas_nssai(text);
            }
    }

    fn on_events_cleared(&mut self) {
        log::debug!("BST Config: Clearing all fields");
        self.current_index = 0;
        self.sib1_fields.clear();
        self.sib2_fields.clear();
        self.sib3_fields.clear();
        self.allowed_nssai.clear();
        self.configured_nssai.clear();
    }

    fn on_metadata_changed(&mut self, metadata: &FileMetadata) {
        self.metadata = metadata.clone();
    }

    fn name(&self) -> &'static str {
        "RAN Config"
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        // Clone metadata to avoid borrow checker issues
        let metadata = self.metadata.clone();
        egui::Window::new("RAN Config")
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
