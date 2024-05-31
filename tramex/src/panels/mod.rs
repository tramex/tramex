//! Module: panels

pub mod about;
pub mod file_upload;
pub mod logical_channels;
pub mod message;
pub mod trame_manager;
pub mod rrc_status;

pub use about::AboutPanel;
pub use file_upload::FileHandler;
pub use logical_channels::LogicalChannels;
pub use rrc_status::LinkPannel;
pub use message::MessageBox;
pub use trame_manager::TrameManager;

use eframe::egui;
use tramex_tools::errors::TramexError;

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
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError>;
}
