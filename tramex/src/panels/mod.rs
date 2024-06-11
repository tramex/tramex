//! Module: panels

pub mod handler_file;
#[cfg(feature = "websocket")]
pub mod handler_ws;
pub mod logical_channels;
pub mod panel_message;
pub mod trame_manager;

pub mod functions_panels;

use eframe::egui;
use tramex_tools::{connector::Connector, data::Data, errors::TramexError};

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

/// Handler are Ui for interface
pub trait Handler {
    /// Render the file upload
    /// # Errors
    /// Return an error if the file contains errors
    fn ui(&mut self, ui: &mut egui::Ui, conn: &mut Connector, new_ctx: egui::Context) -> Result<(), TramexError>;

    /// Render the options
    fn ui_options(&mut self, ui: &mut egui::Ui);
}
