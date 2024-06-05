//! Logical Channels panel
use eframe::egui;
use tramex_tools::data::Data;
use tramex_tools::errors::TramexError;

use super::functions_panels::{make_label, CustomLabelColor};

/// Logical Channels data
#[derive(Default)]
pub struct LogicalChannels {
    /// Current channel
    channel: String,

    /// Current canal
    canal: String,

    /// Current canal message
    canal_msg: String,

    /// Current index
    current_index: usize,

    /// Current hexa
    hex: Vec<u8>,
}

impl LogicalChannels {
    /// Create a new LogicalChannels
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Handle the logic of the panel
    pub fn handle_logic(&mut self) {
        match (self.canal.as_str(), self.canal_msg.as_str()) {
            ("BCCH-BCH", "Master Information Block") => {}
            ("BCCH", "SIB1") => {}
            _ => {
                log::info!("Unknown message");
            }
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

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError> {
        let mut new_index = None;
        {
            // in a closure to avoid borrow checker
            let events = &data.events;
            if self.current_index != data.current_index {
                if let Some(one_log) = events.get(data.current_index) {
                    self.channel = one_log.trace_type.canal.to_owned();
                    self.canal = one_log.trace_type.canal.to_owned();
                    self.canal_msg = one_log.trace_type.canal_msg.to_owned();
                    self.hex = one_log.hexa.to_owned();
                }
                new_index = Some(data.current_index);
            }
        }
        if let Some(idx) = new_index {
            self.handle_logic();
            self.current_index = idx;
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
        Ok(())
    }
}

/// Print a label on the grid
pub fn print_on_grid(ui: &mut egui::Ui, label: &str) {
    ui.vertical_centered(|ui| {
        ui.label(label);
    });
}

/// Convert a number to a boolean
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

        egui::Grid::new("some_unique_id").min_col_width(60.0).show(ui, |ui| {
            make_label(ui, "PCCH", num_to_bool(state & 0x0001), CustomLabelColor::Red);
            make_label(ui, "BCCH", num_to_bool(state & 0x0002), CustomLabelColor::Red);
            make_label(ui, "CCCH", num_to_bool(state & 0x0002), CustomLabelColor::Green);
            make_label(ui, "DCCH", num_to_bool(state & 0x0002), CustomLabelColor::Red);
            make_label(ui, "DTCH", num_to_bool(state & 0x0002), CustomLabelColor::Red);
            make_label(ui, "MCCH", num_to_bool(state & 0x0002), CustomLabelColor::Red);
            make_label(ui, "MTCH", num_to_bool(state & 0x0002), CustomLabelColor::Red);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Canaux Logiques");
            print_on_grid(ui, "----");
            make_label(ui, "CCCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            make_label(ui, "DCCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            make_label(ui, "DTCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            ui.end_row();

            make_label(ui, "PCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            make_label(ui, "BCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            make_label(ui, "DL-SCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            print_on_grid(ui, "");
            make_label(ui, "MCH", num_to_bool(state & 0x0010), CustomLabelColor::Red);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Canaux de Transport");
            print_on_grid(ui, "----");
            make_label(ui, "RACH", num_to_bool(state & 0x0010), CustomLabelColor::Blue);
            make_label(ui, "UL-SCH", num_to_bool(state & 0x0010), CustomLabelColor::Blue);
            ui.end_row();

            make_label(ui, "PDSCH", num_to_bool(state & 0x0010), CustomLabelColor::Blue);
            make_label(ui, "PBCH", num_to_bool(state & 0x0010), CustomLabelColor::Orange);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            make_label(ui, "PDCCH", num_to_bool(state & 0x0010), CustomLabelColor::Orange);
            print_on_grid(ui, "");
            make_label(ui, "PMCH", num_to_bool(state & 0x0010), CustomLabelColor::Orange);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Canaux Physiques");
            print_on_grid(ui, "----");
            make_label(ui, "PRACH", num_to_bool(state & 0x0010), CustomLabelColor::Blue);
            make_label(ui, "PUSCH", num_to_bool(state & 0x0010), CustomLabelColor::Blue);
            make_label(ui, "PUCCH", num_to_bool(state & 0x0010), CustomLabelColor::Orange);
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
