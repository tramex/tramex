//! Module: panels

pub mod bst_config;
pub mod chronograph;
pub mod harq_panel;
pub mod identity;
pub mod logical_channels;
pub mod navigation_panel;
pub mod panel_message;
pub mod power_panel;
pub mod resources_blocks;
pub mod rrc_status;
pub mod trame_manager;

use eframe::egui;

/// Something to view in the demo windows
pub trait PanelView {
    /// Show the UI of the panel
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Shared `egui` memory key used by panels to request navigation to a specific
/// trace index (e.g. clicking an arrow in the HARQ panel). `FrontEnd` polls this
/// value after showing panel windows and forwards it to `Application::navigate_to`.
pub const NAVIGATE_REQUEST_ID: &str = "tramex_panel_navigate_request";
