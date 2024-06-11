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
        color.get_type_channel()
    } else {
        CustomLabelColor::White.get_type_channel()
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
            ("BCCH", "SIB1") | ("BCCH", "SIB") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::BCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("CCCH", "RRC connection request") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_CCCH,
                    transport: TransportChannelsEnum::RACH,
                    physical: PhysicalChannelsEnum::PRACH,
                });
            }
            ("CCCH", "RRC connection setup") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_CCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "RRC connection setup complete") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "DL information transfer") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_DCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "UL information transfer") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "Security mode command") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_DCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "Security mode complete") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "UE capability enquiry") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_DCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "UE capability information") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "RRC connection reconfiguration") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_DCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "RRC connection reconfiguration complete") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "RRC connection reestablishment request") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_CCCH,
                    transport: TransportChannelsEnum::RACH,
                    physical: PhysicalChannelsEnum::PRACH,
                });
            }
            ("CCCH", "RRC connection reestablishment") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_CCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "RRC connection release") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_CCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            _ => {
                log::info!("Unknown message");
            }
        }
    }

    /// Create a label with hover effect for logical channels
    fn make_label_hover_logical(&self, ui: &mut egui::Ui, logical_channel: LogicalChannelsEnum) {
        let is_active = match &self.state {
            Some(state) => state.logical == logical_channel,
            None => false,
        };
        make_label_hover(ui, &logical_channel.to_string(), is_active, logical_channel.get_color());
    }

    /// Create a label with hover effect for transport channels
    fn make_label_hover_transport(&self, ui: &mut egui::Ui, transport_channel: TransportChannelsEnum) {
        let is_active = if let Some(state) = &self.state {
            state.transport == transport_channel
        } else {
            false
        };
        make_label_hover(ui, &transport_channel.to_string(), is_active, transport_channel.get_color());
    }

    /// Create a label with hover effect for physical channels
    fn make_label_hover_physical(&self, ui: &mut egui::Ui, physical_channel: PhysicalChannelsEnum) {
        let is_active = if let Some(state) = &self.state {
            state.physical == physical_channel
        } else {
            false
        };
        make_label_hover(ui, &physical_channel.to_string(), is_active, physical_channel.get_color());
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
#[inline]
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
            self.make_label_hover_logical(ui, LogicalChannelsEnum::PCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::BCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_CCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_DCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::DL_DTCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::MCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::MTCH);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Logical channels");
            print_on_grid(ui, "----");
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_CCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_DCCH);
            self.make_label_hover_logical(ui, LogicalChannelsEnum::UL_DTCH);
            ui.end_row();

            self.make_label_hover_transport(ui, TransportChannelsEnum::PCH);
            self.make_label_hover_transport(ui, TransportChannelsEnum::BCH);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            self.make_label_hover_transport(ui, TransportChannelsEnum::DL_SCH);
            print_on_grid(ui, "");
            self.make_label_hover_transport(ui, TransportChannelsEnum::MCH);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Transport channels");
            print_on_grid(ui, "----");
            self.make_label_hover_transport(ui, TransportChannelsEnum::RACH);
            self.make_label_hover_transport(ui, TransportChannelsEnum::UL_SCH);
            ui.end_row();

            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PDSCH);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PBCH);
            print_on_grid(ui, "");
            print_on_grid(ui, "");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PDCCH);
            print_on_grid(ui, "");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PMCH);
            print_on_grid(ui, "----");
            print_on_grid(ui, "Physical channels");
            print_on_grid(ui, "----");
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PRACH);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PUSCH);
            self.make_label_hover_physical(ui, PhysicalChannelsEnum::PUCCH);
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
