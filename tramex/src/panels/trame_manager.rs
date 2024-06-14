//! TrameManager
use eframe::egui;
use tramex_tools::data::Data;
use tramex_tools::interface::layer::{LayerLogLevel, Layers};

#[derive(serde::Deserialize, serde::Serialize)]
/// TrameManager
pub struct TrameManager {
    /// Layers
    pub layers_list: Layers,
    /// boolean to get more log
    pub should_get_more_log: bool,
}

impl TrameManager {
    /// Create a new TrameManager
    pub fn new() -> Self {
        Self {
            layers_list: Layers::new(),
            should_get_more_log: false,
        }
    }
}

impl Default for TrameManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrameManager {
    /// Show the options
    pub fn show_options(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Options", |ui| {
            checkbox(ui, &mut self.layers_list.phy, "PHY");
            checkbox(ui, &mut self.layers_list.mac, "MAC");
            checkbox(ui, &mut self.layers_list.rlc, "RLC");
            checkbox(ui, &mut self.layers_list.pdcp, "PDCP");
            checkbox(ui, &mut self.layers_list.rrc, "RRC");
            checkbox(ui, &mut self.layers_list.nas, "NAS");
            checkbox(ui, &mut self.layers_list.s72, "S72");
            checkbox(ui, &mut self.layers_list.s1ap, "S1AP");
            checkbox(ui, &mut self.layers_list.ngap, "NGAP");
            checkbox(ui, &mut self.layers_list.gtpu, "GTPU");
            checkbox(ui, &mut self.layers_list.x2ap, "X2AP");
            checkbox(ui, &mut self.layers_list.xnap, "XnAP");
            checkbox(ui, &mut self.layers_list.m2ap, "M2AP");
            checkbox(ui, &mut self.layers_list.lppa, "LPPa");
            checkbox(ui, &mut self.layers_list.nrppa, "NRPPa");
            checkbox(ui, &mut self.layers_list.trx, "TRX");
        });
    }

    /// Show the controls
    pub fn show_controls(&mut self, ui: &mut egui::Ui, data: &mut Data, is_full_read: bool) {
        ui.add_enabled_ui(!is_full_read, |ui| {
            if ui.button("More").clicked() {
                log::debug!("More");
                self.should_get_more_log = true;
            }
        });
        let is_enabled = if !data.events.is_empty() && data.events.len() - 1 == data.current_index {
            !is_full_read
        } else {
            true
        };
        ui.add_enabled_ui(is_enabled, |ui| {
            let text = if data.events.is_empty() { "Start" } else { "Next" };
            if ui.button(text).clicked() {
                log::debug!("Next {}", text);
                if data.events.len() > data.current_index + 1 {
                    data.current_index += 1;
                } else {
                    self.should_get_more_log = true;
                }
                // preloading
                if !data.events.is_empty() && data.current_index == (data.events.len() - 1) {
                    log::debug!("Preloading");
                    self.should_get_more_log = true;
                }
            }
        });
        ui.add_enabled_ui(data.current_index > 0, |ui| {
            if ui.button("Previous").clicked() {
                log::debug!("Previous");
                if data.current_index > 0 {
                    data.current_index -= 1;
                }
            }
        });
    }
}

/// Create a checkbox
fn checkbox(ui: &mut egui::Ui, string: &mut LayerLogLevel, text: &str) {
    let mut checked = match string {
        LayerLogLevel::Debug => true,
        LayerLogLevel::Warn => false,
    };
    if ui.checkbox(&mut checked, text).changed() {
        if checked {
            *string = LayerLogLevel::Debug;
        } else {
            *string = LayerLogLevel::Warn;
        };
    };
}
