//! Module: panels

pub mod file_upload;
pub mod logical_channels;
pub mod panel_message;
pub mod rrc_status;
pub mod trame_manager;

pub mod functions_panels;

use eframe::egui;
use tramex_tools::{data::Data, errors::TramexError};

/// Something to view in the demo windows
pub trait PanelView {
    /// Show the UI of the panel
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Something to view
pub trait PanelController {
    /// Is the demo enabled for this integration?
    fn is_enabled(&self, _ctx: &egui::Context) -> bool {
        true
    }

    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// `&'static` so we can also use it as a key to store open/close state.
    fn window_title(&self) -> &'static str;

    /// Show windows, etc
    /// # Errors
    /// Return an error if the panel can't be shown
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError>;
}
