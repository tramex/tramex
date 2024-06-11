//! Logical Channels panel

use eframe::egui;
use tramex_tools::data::AdditionalInfos;
use tramex_tools::data::Data;
use tramex_tools::errors::TramexError;

use super::functions_panels::LogicalChannelsEnum;
use super::functions_panels::PhysicalChannelsEnum;
use super::functions_panels::TransportChannelsEnum;
use super::functions_panels::{make_label, CustomLabelColor};

/// Upgraded version of make_label function with explanation of the channel color when hovering on it
pub fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) {
    make_label(ui, label, show, color.clone()).on_hover_text_at_pointer(if show {
        match color {
            CustomLabelColor::Red => "Broadcast channel",
            CustomLabelColor::Blue => "Common channel",
            CustomLabelColor::Green => "Traffic channel",
            CustomLabelColor::Orange => "Dedicated channel",
            CustomLabelColor::White => "This channel is currently unused",
        }
    } else {
        "This channel is currently unused"
    });
}

/// Logical Channels data
#[derive(Default)]
pub struct LogicalChannels {
    /// Current canal
    canal: String,

    /// Current canal message
    canal_msg: String,

    /// Current index
    current_index: usize,

    /// channel state : which logical channels to switch on
    state: Option<ChannelState>,
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
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::BCH,
                    physical: PhysicalChannelsEnum::PBCH,
                });
            }
            ("BCCH", "SIB1") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("BCCH", "SIB") => {
                self.state = Some(ChannelState {
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

    fn make_label_hover_logical(&self, ui: &mut egui::Ui, logical_channel: LogicalChannelsEnum, color: CustomLabelColor) {
        let is_active = match &self.state {
            Some(state) => state.logical == logical_channel,
            None => false,
        };
        make_label_hover(ui, &logical_channel.to_string(), is_active, color);
    }

    fn make_label_hover_transport(
        &self,
        ui: &mut egui::Ui,
        transport_channel: TransportChannelsEnum,
        color: CustomLabelColor,
    ) {
        let is_active = if let Some(state) = &self.state {
            state.transport == transport_channel
        } else {
            false
        };
        make_label_hover(ui, &transport_channel.to_string(), is_active, color);
    }

    fn make_label_hover_physical(&self, ui: &mut egui::Ui, physical_channel: PhysicalChannelsEnum, color: CustomLabelColor) {
        let is_active = if let Some(state) = &self.state {
            state.physical == physical_channel
        } else {
            false
        };
        make_label_hover(ui, &physical_channel.to_string(), is_active, color);
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
        if data.is_different_index(self.current_index) {
            if let Some(one_trace) = data.get_current_trace() {
                match &one_trace.additional_infos {
                    AdditionalInfos::RRCInfos(infos) => {
                        self.canal = infos.canal.to_owned();
                        self.canal_msg = infos.canal_msg.to_owned();
                    }
                }
            }
            self.current_index = data.current_index;
            self.handle_logic();
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

/// Struct that contains the three logical channels associated to the current message
#[derive(PartialEq)]
struct ChannelState {
    /// Logical channel of current message
    logical: LogicalChannelsEnum,

    /// Transport channel of current message
    transport: TransportChannelsEnum,

    /// Physical channel of current message
    physical: PhysicalChannelsEnum,
}

impl super::PanelView for LogicalChannels {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("some_unique_id").min_col_width(60.0).show(ui, |ui| {
            self.make_label_hover_logical(ui, LogicalChannelsEnum::PCCH, CustomLabelColor::Blue);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::BCCH, CustomLabelColor::Red);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_CCCH, CustomLabelColor::Blue);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_DCCH, CustomLabelColor::Orange);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_DTCH, CustomLabelColor::Green);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::MCCH, CustomLabelColor::Blue);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::MTCH, CustomLabelColor::Green);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Logical channels");
            print_on_grid(ui, "----");
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_CCCH, CustomLabelColor::Blue);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_DCCH, CustomLabelColor::Orange);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_DTCH, CustomLabelColor::Green);
            ui.end_row();

            self.make_label_hover_transport(ui, TransportChannelsEnum::PCH, CustomLabelColor::Blue);
            self.make_label_hover_transport(ui, TransportChannelsEnum::BCH, CustomLabelColor::Red);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            self.make_label_hover_transport(ui, TransportChannelsEnum::DL_SCH, CustomLabelColor::Green);
            print_on_grid(ui, "");
            self.make_label_hover_transport(ui, TransportChannelsEnum::MCH, CustomLabelColor::Green);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Transport channels");
            print_on_grid(ui, "----");
            self.make_label_hover_transport(ui, TransportChannelsEnum::RACH, CustomLabelColor::Blue);
            self.make_label_hover_transport(ui, TransportChannelsEnum::UL_SCH, CustomLabelColor::Green);
            ui.end_row();

            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PDSCH, CustomLabelColor::Green);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PBCH, CustomLabelColor::Red);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PDCCH, CustomLabelColor::Orange);
            print_on_grid(ui, "");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PMCH, CustomLabelColor::Green);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Physical channels");
            print_on_grid(ui, "----");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PRACH, CustomLabelColor::Blue);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PUSCH, CustomLabelColor::Green);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PUCCH, CustomLabelColor::Orange);
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
}
