//! Module: panels

use tramex_tools::{connector::Connector, errors::TramexError};

pub mod handler_file;
#[cfg(feature = "websocket")]
pub mod handler_ws;

/// Handler are Ui for interface
pub trait Handler {
    /// Render the file upload
    /// # Errors
    /// Return an error if the file contains errors
    fn ui(&mut self, ui: &mut egui::Ui, conn: &mut Connector, new_ctx: egui::Context) -> Result<(), TramexError>;

    /// Render the options
    fn ui_options(&mut self, ui: &mut egui::Ui);
}
