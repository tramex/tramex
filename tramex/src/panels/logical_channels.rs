use eframe::egui::{self, Color32, TextFormat};
use std::cell::RefCell;
use std::rc::Rc;
use tramex_tools::connector::Connector;

pub struct LogicalChannels {
    data: Rc<RefCell<Connector>>,
    channel: String,
}

impl LogicalChannels {
    pub fn new(ref_data: Rc<RefCell<Connector>>) -> Self {
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
            let events = &borrowed.data.events;
            if let Some(one_log) = events.get(borrowed.data.current_index) {
                self.channel = one_log.trace_type.canal.to_owned();
            }
        }
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .resizable([true, false])
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
    }
}

pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: &str) {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let mut format = TextFormat {
        ..Default::default()
    };
    if show {
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

fn num_to_bool(num: u32) -> bool {
    num == 1
}

impl super::PanelView for LogicalChannels {
    fn ui(&mut self, ui: &mut egui::Ui) {
        //TODO CHANGE
        let state = match self.channel.as_str() {
            "PCCH" => 0x00000001,
            _ => 0x00000000,
        };

        egui::Grid::new("some_unique_id")
            .min_col_width(60.0)
            .show(ui, |ui| {
                make_label(ui, "PCCH", num_to_bool(state & 0x0001), "red");
                make_label(ui, "BCCH", num_to_bool(state & 0x0002), "red");
                make_label(ui, "CCCH", num_to_bool(state & 0x0002), "green");
                make_label(ui, "DCCH", num_to_bool(state & 0x0002), "red");
                make_label(ui, "DTCH", num_to_bool(state & 0x0002), "red");
                make_label(ui, "MCCH", num_to_bool(state & 0x0002), "red");
                make_label(ui, "MTCH", num_to_bool(state & 0x0002), "red");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux Logiques");
                print_on_grid(ui, "----");
                make_label(ui, "CCCH", num_to_bool(state & 0x0010), "red");
                make_label(ui, "DCCH", num_to_bool(state & 0x0010), "red");
                make_label(ui, "DTCH", num_to_bool(state & 0x0010), "red");
                ui.end_row();

                make_label(ui, "PCH", num_to_bool(state & 0x0010), "red");
                make_label(ui, "BCH", num_to_bool(state & 0x0010), "red");
                print_on_grid(ui, "");
                print_on_grid(ui, "");
                make_label(ui, "DL-SCH", num_to_bool(state & 0x0010), "red");
                print_on_grid(ui, "");
                make_label(ui, "MCH", num_to_bool(state & 0x0010), "red");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux de Transport");
                print_on_grid(ui, "----");
                make_label(ui, "RACH", num_to_bool(state & 0x0010), "blue");
                make_label(ui, "UL-SCH", num_to_bool(state & 0x0010), "blue");
                ui.end_row();

                make_label(ui, "PDSCH", num_to_bool(state & 0x0010), "blue");
                make_label(ui, "PBCH", num_to_bool(state & 0x0010), "orange");
                print_on_grid(ui, "");
                print_on_grid(ui, "");
                make_label(ui, "PDCCH", num_to_bool(state & 0x0010), "orange");
                print_on_grid(ui, "");
                make_label(ui, "PMCH", num_to_bool(state & 0x0010), "orange");
                print_on_grid(ui, "----");
                print_on_grid(ui, "Canaux Logiques");
                print_on_grid(ui, "----");
                make_label(ui, "PRACH", num_to_bool(state & 0x0010), "blue");
                make_label(ui, "PUSCH", num_to_bool(state & 0x0010), "blue");
                make_label(ui, "PUCCH", num_to_bool(state & 0x0010), "orange");
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
