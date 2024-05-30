//! Logical Channels panel
use asn1_codecs::{uper::UperCodec, PerCodecData};
use eframe::egui::{self, Color32, TextFormat};
use std::cell::RefCell;
use std::rc::Rc;
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;
use types_lte_3gpp::spec_rrc;

/// Logical Channels data
pub struct LogicalChannels {
    /// Reference to the data
    data: Rc<RefCell<Connector>>,

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
    pub fn new(ref_data: Rc<RefCell<Connector>>) -> Self {
        Self {
            data: ref_data,
            channel: "".to_string(),
            canal: "".to_string(),
            canal_msg: "".to_string(),
            current_index: 0,
            hex: Vec::new(),
        }
    }

    /// Handle the logic of the panel
    pub fn handle_logic(&mut self) {
        match (self.canal.as_str(), self.canal_msg.as_str()) {
            ("BCCH-BCH", "Master Information Block") => {
                let mut codec_data = PerCodecData::from_slice_uper(&self.hex);
                let sib1 = spec_rrc::BCCH_BCH_Message::uper_decode(&mut codec_data);
                if let Ok(res) = sib1 {
                    log::info!("{:?}", res);
                }
            }
            ("BCCH", "SIB1") => {
                let mut codec_data = PerCodecData::from_slice_uper(&self.hex);
                let sib1 = spec_rrc::BCCH_BCH_Message::uper_decode(&mut codec_data);
                if let Ok(res) = sib1 {
                    log::info!("{:?}", res);
                }
            }
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

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        let mut new_index = None;
        {
            // in a closure to avoid borrow checker
            let borrowed = &self.data.borrow();
            let events = &borrowed.data.events;
            if self.current_index != borrowed.data.current_index {
                if let Some(one_log) = events.get(borrowed.data.current_index) {
                    self.channel = one_log.trace_type.canal.to_owned();
                    self.canal = one_log.trace_type.canal.to_owned();
                    self.canal_msg = one_log.trace_type.canal_msg.to_owned();
                    self.hex = one_log.hexa.to_owned();
                }
                new_index = Some(borrowed.data.current_index);
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

/// Create a label with a background color
pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: &str) {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let mut format = TextFormat { ..Default::default() };
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
            print_on_grid(ui, "Canaux Physiques");
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
