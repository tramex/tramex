//! Module: panels

pub mod bst_config;
pub mod chronograph;
pub mod harq_panel;
pub mod identity;
pub mod logical_channels;
pub mod navigation_panel;
pub mod panel_message;
pub mod resources_blocks;
pub mod rrc_status;
pub mod trame_manager;

use eframe::egui;

/// Something to view in the demo windows
pub trait PanelView {
    /// Show the UI of the panel
    fn ui(&mut self, ui: &mut egui::Ui);
}
