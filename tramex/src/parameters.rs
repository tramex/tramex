//! Runtime parameters shared state
//!
//! User-tunable runtime parameters, editable in the Settings window
//! and persisted between sessions.

/// Default maximum number of events requested per `log_get` request
#[cfg(feature = "websocket")]
const DEFAULT_WS_REQUEST_MAX_SIZE: u64 = tramex_tools::interface::websocket::ws_connection::DEFAULT_ASKING_SIZE_MAX;

/// Default maximum number of events requested per `log_get` request
#[cfg(not(feature = "websocket"))]
const DEFAULT_WS_REQUEST_MAX_SIZE: u64 = 1024;

/// Default number of hours back the server is asked for logs
#[cfg(feature = "websocket")]
const DEFAULT_WS_HISTORY_HOURS: i64 = tramex_tools::interface::websocket::ws_connection::DEFAULT_HISTORY_HOURS;

/// Default number of hours back the server is asked for logs
#[cfg(not(feature = "websocket"))]
const DEFAULT_WS_HISTORY_HOURS: i64 = 12;

/// Runtime parameters editable in the Settings window
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Parameters {
    /// Maximum number of arrows kept in the HARQ panel buffer
    pub harq_max_arrows: usize,

    /// Maximum number of arrows kept in the Chronograph RAN panel
    pub chronograph_max_arrows: usize,

    /// Maximum number of arrows kept in the Chronograph MAC-RLC-PDCP panel
    pub chronograph_mac_max_arrows: usize,

    /// Number of events parsed per batch when loading a file
    pub file_batch_size: usize,

    /// Maximum number of events requested per `log_get` request
    pub ws_request_max_size: u64,

    /// How many hours back the server is asked for logs
    pub ws_history_hours: i64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            harq_max_arrows: crate::panels::harq_panel::DEFAULT_MAX_ARROWS,
            chronograph_max_arrows: crate::panels::chronograph::DEFAULT_MAX_ARROWS,
            chronograph_mac_max_arrows: crate::panels::chronograph_mac::DEFAULT_MAX_ARROWS,
            file_batch_size: tramex_tools::interface::interface_file::file_handler::BATCH_SIZE,
            ws_request_max_size: DEFAULT_WS_REQUEST_MAX_SIZE,
            ws_history_hours: DEFAULT_WS_HISTORY_HOURS,
        }
    }
}

impl Parameters {
    /// Show the parameters settings UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let colors = crate::theme::ThemeColors::get(ui);

        ui.label(egui::RichText::new("Panels").strong());
        ui.horizontal(|ui| {
            ui.label("HARQ max arrows:");
            ui.add(egui::DragValue::new(&mut self.harq_max_arrows).range(10..=1000))
                .on_hover_text("Maximum number of arrows kept in the HARQ panel buffer");
        });
        ui.horizontal(|ui| {
            ui.label("Chronograph RAN max arrows:");
            ui.add(egui::DragValue::new(&mut self.chronograph_max_arrows).range(10..=1000))
                .on_hover_text("Maximum number of arrows kept in the Chronograph RAN panel buffer");
        });
        ui.horizontal(|ui| {
            ui.label("Chronograph MAC max arrows:");
            ui.add(egui::DragValue::new(&mut self.chronograph_mac_max_arrows).range(10..=1000))
                .on_hover_text("Maximum number of arrows kept in the Chronograph MAC-RLC-PDCP panel buffer");
        });
        ui.add_space(4.0);

        ui.label(egui::RichText::new("File").strong());
        ui.horizontal(|ui| {
            ui.label("Batch size:");
            ui.add(egui::DragValue::new(&mut self.file_batch_size).range(10..=10_000))
                .on_hover_text("Number of events parsed per batch when loading a file");
        });
        ui.add_space(4.0);

        ui.label(egui::RichText::new("WebSocket").strong());
        ui.horizontal(|ui| {
            ui.label("Max events per request:");
            ui.add(egui::DragValue::new(&mut self.ws_request_max_size).range(64..=4096))
                .on_hover_text("Maximum number of events requested per log_get request");
        });
        ui.horizontal(|ui| {
            ui.label("History window (hours):");
            ui.add(egui::DragValue::new(&mut self.ws_history_hours).range(1..=168))
                .on_hover_text("How far back the server is asked for logs");
        });
        ui.label(
            egui::RichText::new("Applies to the next request")
                .small()
                .color(colors.text_weak),
        );

        ui.add_space(8.0);
        if ui.button("Reset to defaults").clicked() {
            *self = Self::default();
        }
    }
}
