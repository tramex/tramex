use crate::Data;
use eframe::egui::{self, Color32, TextFormat};
use std::cell::RefCell;
use std::rc::Rc;

pub struct LogicalChannels {
    data: Rc<RefCell<Data>>,
    channel: String,
}

impl LogicalChannels {
    pub fn new(ref_data: Rc<RefCell<Data>>) -> Self {
        Self {
            data: ref_data,
            channel: "".to_string(),
        }
    }
}

impl super::PanelController for LogicalChannels {
    fn name(&self) -> &'static str {
        "Canaux logiques"
    }
    fn window_title(&self) -> &'static str {
        "Téléphone - Canaux logiques (couche 3)"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        {
            // in a closure to avoid borrow checker
            let borrowed = &self.data.borrow();
            let events = &borrowed.events;
            if let Some(one_log) = events.get(borrowed.current_index) {
                if let Some(log) = &one_log.channel {
                    self.channel = log.to_owned();
                }
            }
        }
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

pub fn make_label(ui: &mut egui::Ui, label: &str, state: &str, color: &str) {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let mut format = TextFormat {
        ..Default::default()
    };
    if label == state {
        format.color = Color32::BLACK;
        format.background = match color {
            "red" => Color32::from_rgb(255, 84, 84),
            "blue" => Color32::from_rgb(68, 143, 255),
            "orange" => Color32::from_rgb(255, 181, 68),
            "green" => Color32::from_rgb(90, 235, 100),
            _ => Color32::from_rgb(90, 235, 100),
        };
    }

    job.append(label, 0.0, format);
    ui.vertical_centered(|ui| {
        ui.label(job);
    });
}

pub fn print_on_grid(ui: &mut egui::Ui, label: &str) {
    ui.vertical_centered(|ui| {
        ui.label(label);
    });
}

impl super::PanelView for LogicalChannels {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("some_unique_id")
            .max_col_width(120.0)
            .show(ui, |ui| {
                make_label(ui, "PCCH", &self.channel, "red");
                make_label(ui, "BCCH", &self.channel, "red");
                make_label(ui, "CCCH", &self.channel, "green");
                make_label(ui, "DCCH", &self.channel, "red");
                make_label(ui, "DTCH", &self.channel, "red");
                make_label(ui, "MCCH", &self.channel, "red");
                make_label(ui, "MTCH", &self.channel, "red");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux Logiques");
                print_on_grid(ui, "----");
                make_label(ui, "CCCH", &self.channel, "red");
                make_label(ui, "DCCH", &self.channel, "red");
                make_label(ui, "DTCH", &self.channel, "red");
                ui.end_row();

                make_label(ui, "PCH", &self.channel, "red");
                make_label(ui, "BCH", &self.channel, "red");
                print_on_grid(ui, "");
                print_on_grid(ui, "");
                make_label(ui, "DL-SCH", &self.channel, "red");
                print_on_grid(ui, "");
                make_label(ui, "MCH", &self.channel, "red");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux de Transport");
                print_on_grid(ui, "----");
                make_label(ui, "RACH", &self.channel, "blue");
                make_label(ui, "UL-SCH", &self.channel, "blue");
                ui.end_row();

                make_label(ui, "PDSCH", &self.channel, "blue");
                make_label(ui, "PBCH", &self.channel, "orange");
                print_on_grid(ui, "");
                print_on_grid(ui, "");
                make_label(ui, "PDCCH", &self.channel, "orange");
                print_on_grid(ui, "");
                make_label(ui, "PMCH", &self.channel, "orange");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux Logiques");
                print_on_grid(ui, "----");
                make_label(ui, "PRACH", &self.channel, "blue");
                make_label(ui, "PUSCH", &self.channel, "blue");
                make_label(ui, "PUCCH", &self.channel, "orange");
                ui.end_row();

                print_on_grid(ui, "----");
                print_on_grid(ui, "----");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Downlink");
                print_on_grid(ui, "----");
                print_on_grid(ui, "----");
                print_on_grid(ui, "----");
                print_on_grid(ui, "");
                print_on_grid(ui, "Technologie : LTE");
                print_on_grid(ui, "");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Uplink");
                print_on_grid(ui, "----");
                ui.end_row();
            });
    }
}
