use crate::{display_log, Data};
use eframe::egui;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MessageBox {
    data: Rc<RefCell<Data>>,
}

impl MessageBox {
    pub fn new(ref_data: Rc<RefCell<Data>>) -> Self {
        Self { data: ref_data }
    }
}

impl super::PanelController for MessageBox {
    fn name(&self) -> &'static str {
        "Messages"
    }

    fn window_title(&self) -> &'static str {
        "Socket Message"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
    }
}

impl super::PanelView for MessageBox {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Received events:");
        let borrowed = &self.data.borrow();
        let events = &borrowed.events;
        ui.horizontal(|ui| {
            ui.label(format!("Received events: {}", events.len()));
        });
        ui.label(format!("Current msg index: {}", borrowed.current_index));

        if let Some(one_log) = events.get(borrowed.current_index) {
            display_log(ui, &one_log);
        }
    }
}
