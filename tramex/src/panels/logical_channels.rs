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

    state: Option<LogicalChannelState>,
}

impl LogicalChannels {
    /// Create a new LogicalChannels
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Handle the logic of the panel
    pub fn handle_logic(&mut self) {
        match (self.canal.as_str(), self.canal_msg.as_str()) {
            ("BCCH-BCH", "Master Information Block") => {
                self.state = Some(LogicalChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::BCH,
                    physical: PhysicalChannelsEnum::PBCH,
                });
            }
            ("BCCH", "SIB1") => {
                self.state = Some(LogicalChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("BCCH", "SIB") => {
                self.state = Some(LogicalChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            _ => {
                log::info!("Unknown message");
            }
        }
    }
}

impl super::PanelController for LogicalChannels {
    fn name(&self) -> &'static str {
        "Logical channels"
    }
    fn window_title(&self) -> &'static str {
        "Mobile Phone - Logical channels (layer 3)"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError> {
        let mut new_index = None;
        {
            // in a closure to avoid borrow checker
            let events = &data.events;
            if self.current_index != data.current_index {
                if let Some(one_log) = data.get_current_trace() {
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
#[derive(PartialEq)]
/// Enumerate all types of logical channels in LTE technology
pub enum LogicalChannelsEnum {
    /// Paging Control Channel
    PCCH,

    /// Broadcast Control Channel
    BCCH,

    ///  Downlink Common Control Channel
    DL_CCCH,

    /// Downlink Dedicated Control Channel
    DL_DCCH,

    /// Downlink Dedicated Traffic Channel
    DL_DTCH,

    /// Multicast Control Channel
    MCCH,

    /// Multicast Traffic Channel
    MTCH,

    /// Uplink Common Control Channel
    UL_CCCH,

    /// Uplink Dedicated Control Channel
    UL_DCCH,

    /// Uplink Dedicated Traffic Channel
    UL_DTCH,
}
#[derive(PartialEq)]
/// Enumerate all types of transport channels in LTE technology
pub enum TransportChannelsEnum {
    /// Paging Channel
    PCH,

    /// Broadcast Channel
    BCH,

    /// Downlink Shared Channel
    DL_SCH,

    /// Multicast Channel
    MCH,

    /// Random Access Channel
    RACH,

    /// Uplink Shared Channel
    UL_SCH,
}
#[derive(PartialEq)]
/// Enumerate all types of physical channels in LTE technology
pub enum PhysicalChannelsEnum {
    /// Physical Downlink Shared Channel
    PDSCH,

    /// Physical Broadcast Channel
    PBCH,

    /// Physical Downlink Control Channel
    PDCCH,

    /// Physical Multicast Channel
    PMCH,

    /// Physical Random Access Channel
    PRACH,

    /// Physical Uplink Shared Channel
    PUSCH,

    /// Physical Uplink Control Channel
    PUCCH,
}
#[derive(PartialEq)]
struct LogicalChannelState {
    logical: LogicalChannelsEnum,
    transport: TransportChannelsEnum,
    physical: PhysicalChannelsEnum,
}

impl super::PanelView for LogicalChannels {
    fn ui(&mut self, ui: &mut egui::Ui) {
        match &self.state {
            Some(state) => {
                egui::Grid::new("some_unique_id").min_col_width(60.0).show(ui, |ui| {
                    make_label(ui, "PCCH", state.logical == LogicalChannelsEnum::PCCH, CustomLabelColor::Blue);
                    make_label(ui, "BCCH", state.logical == LogicalChannelsEnum::BCCH, CustomLabelColor::Red);
                    make_label(
                        ui,
                        "CCCH",
                        state.logical == LogicalChannelsEnum::DL_CCCH,
                        CustomLabelColor::Blue,
                    );
                    make_label(
                        ui,
                        "DCCH",
                        state.logical == LogicalChannelsEnum::DL_DCCH,
                        CustomLabelColor::Orange,
                    );
                    make_label(
                        ui,
                        "DTCH",
                        state.logical == LogicalChannelsEnum::DL_DTCH,
                        CustomLabelColor::Green,
                    );
                    make_label(ui, "MCCH", state.logical == LogicalChannelsEnum::MCCH, CustomLabelColor::Blue);
                    make_label(
                        ui,
                        "MTCH",
                        state.logical == LogicalChannelsEnum::MTCH,
                        CustomLabelColor::Green,
                    );
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "Logical channels");
                    print_on_grid(ui, "----");
                    make_label(
                        ui,
                        "CCCH",
                        state.logical == LogicalChannelsEnum::UL_CCCH,
                        CustomLabelColor::Blue,
                    );
                    make_label(
                        ui,
                        "DCCH",
                        state.logical == LogicalChannelsEnum::UL_DCCH,
                        CustomLabelColor::Orange,
                    );
                    make_label(
                        ui,
                        "DTCH",
                        state.logical == LogicalChannelsEnum::UL_DTCH,
                        CustomLabelColor::Green,
                    );
                    ui.end_row();

                    make_label(
                        ui,
                        "PCH",
                        state.transport == TransportChannelsEnum::PCH,
                        CustomLabelColor::Blue,
                    );
                    make_label(
                        ui,
                        "BCH",
                        state.transport == TransportChannelsEnum::BCH,
                        CustomLabelColor::Red,
                    );
                    print_on_grid(ui, "");
                    print_on_grid(ui, "");
                    make_label(
                        ui,
                        "DL-SCH",
                        state.transport == TransportChannelsEnum::DL_SCH,
                        CustomLabelColor::Green,
                    );
                    print_on_grid(ui, "");
                    make_label(
                        ui,
                        "MCH",
                        state.transport == TransportChannelsEnum::MCH,
                        CustomLabelColor::Green,
                    );
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "Transport channels");
                    print_on_grid(ui, "----");
                    make_label(
                        ui,
                        "RACH",
                        state.transport == TransportChannelsEnum::RACH,
                        CustomLabelColor::Blue,
                    );
                    make_label(
                        ui,
                        "UL-SCH",
                        state.transport == TransportChannelsEnum::UL_SCH,
                        CustomLabelColor::Green,
                    );
                    ui.end_row();

                    make_label(
                        ui,
                        "PDSCH",
                        state.physical == PhysicalChannelsEnum::PDSCH,
                        CustomLabelColor::Green,
                    );
                    make_label(
                        ui,
                        "PBCH",
                        state.physical == PhysicalChannelsEnum::PBCH,
                        CustomLabelColor::Red,
                    );
                    print_on_grid(ui, "");
                    print_on_grid(ui, "");
                    make_label(
                        ui,
                        "PDCCH",
                        state.physical == PhysicalChannelsEnum::PDCCH,
                        CustomLabelColor::Orange,
                    );
                    print_on_grid(ui, "");
                    make_label(
                        ui,
                        "PMCH",
                        state.physical == PhysicalChannelsEnum::PMCH,
                        CustomLabelColor::Green,
                    );
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "Physical channels");
                    print_on_grid(ui, "----");
                    make_label(
                        ui,
                        "PRACH",
                        state.physical == PhysicalChannelsEnum::PRACH,
                        CustomLabelColor::Blue,
                    );
                    make_label(
                        ui,
                        "PUSCH",
                        state.physical == PhysicalChannelsEnum::PUSCH,
                        CustomLabelColor::Green,
                    );
                    make_label(
                        ui,
                        "PUCCH",
                        state.physical == PhysicalChannelsEnum::PUCCH,
                        CustomLabelColor::Orange,
                    );
                    ui.end_row();

                    print_on_grid(ui, "----");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "Downlink");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "");
                    print_on_grid(ui, "Technology : LTE");
                    print_on_grid(ui, "");
                    print_on_grid(ui, "----");
                    print_on_grid(ui, "Uplink");
                    print_on_grid(ui, "----");
                    ui.end_row();
                });
            }
            None => {}
        }
    }
}
