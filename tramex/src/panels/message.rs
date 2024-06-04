//! Message panel
use crate::display_log;
use eframe::egui;
use std::cell::RefCell;
use std::rc::Rc;
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;

/// Message box
pub struct MessageBox {
    /// Reference to the data
    data: Rc<RefCell<Connector>>,

    /// show full message
    show_full: bool,
}

impl MessageBox {
    /// Create a new MessageBox
    pub fn new(ref_data: Rc<RefCell<Connector>>) -> Self {
        Self {
            data: ref_data,
            show_full: false,
        }
    }
}

impl super::PanelController for MessageBox {
    fn name(&self) -> &'static str {
        "Messages"
    }

    fn window_title(&self) -> &'static str {
        "Current Message"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
        Ok(())
    }
}

impl super::PanelView for MessageBox {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Received events:");
        ui.checkbox(&mut self.show_full, "Show full message");
        let borrowed = &self.data.borrow();
        let events = &borrowed.data.events;
        ui.horizontal(|ui| {
            ui.label(format!("Received events: {}", events.len()));
        });
        let current_index = borrowed.data.current_index;

        if let Some(one_log) = events.get(current_index) {
            ui.label(format!("Current msg index: {}", current_index + 1));
            display_log(ui, one_log, self.show_full);
        }
    }
}
