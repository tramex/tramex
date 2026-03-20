//! Module: panels

pub mod logical_channels;
pub mod navigation_panel;
pub mod panel_message;
pub mod rrc_status;
pub mod bst_config;
pub mod chronograph;
pub mod trame_manager;
pub mod identity;
pub mod ressources_blocks;
pub mod harq_panel;

pub mod functions_panels;

use eframe::egui;

/// Something to view in the demo windows
pub trait PanelView {
    /// Show the UI of the panel
    fn ui(&mut self, ui: &mut egui::Ui);
}
