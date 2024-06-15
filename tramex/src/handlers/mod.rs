//! Module: handlers

use tramex_tools::{data::Data, errors::TramexError, interface::layer::Layers};

pub mod handler_file;
#[cfg(feature = "websocket")]
pub mod handler_ws;

/// Handler are Ui for interface
pub trait Handler {
    /// Render the file upload
    /// Return true if close needed
    /// # Errors
    /// Return an error if the file contains errors
    fn ui(&mut self, ui: &mut egui::Ui, data: &mut Data, new_ctx: egui::Context) -> Result<bool, TramexError>;

    /// Render the options
    fn ui_options(&mut self, ui: &mut egui::Ui);

    /// Get more data depending on the interface
    /// # Errors
    /// Return an error if the interface is not set
    fn get_more_data(&mut self, layer_list: Layers, data: &mut Data) -> Result<(), Vec<TramexError>>;

    /// Try to receive data
    /// # Errors
    /// Return an error if the interface is not set
    fn try_recv(&mut self, data: &mut Data) -> Result<(), Vec<TramexError>>;

    /// Close the interface
    /// # Errors
    /// Return an error closing fails
    fn close(&mut self) -> Result<(), TramexError>;

    /// Show the available options
    fn show_available(&self, ui: &mut egui::Ui);

    /// Check if the read is full
    fn is_full_read(&self) -> bool;

    /// Check if the interface is present
    fn is_interface(&self) -> bool;

    /// Check if the interface is available
    fn is_interface_available(&self) -> bool;
}
