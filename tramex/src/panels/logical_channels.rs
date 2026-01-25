//! Logical Channels panel

use crate::ChannelColors;
use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::PanelView;
use eframe::egui;
use tramex_tools::data::{AdditionalInfos, Trace};
use tramex_tools::errors::TramexError;
use tramex_tools::interface::parse_config::Technology;

use super::functions_panels::LogicalChannelsEnum;
use super::functions_panels::PhysicalChannelsEnum;
use super::functions_panels::TransportChannelsEnum;
use super::functions_panels::make_label;
use egui::Color32;

/// Get channel type description for a color (for hover text)
fn get_channel_type(color: Color32) -> &'static str {
    if color == ChannelColors::RED {
        "Broadcast channel"
    } else if color == ChannelColors::BLUE {
        "Common channel"
    } else if color == ChannelColors::GREEN {
        "Traffic channel"
    } else if color == ChannelColors::ORANGE {
        "Dedicated channel"
    } else {
        "This channel is currently unused"
    }
}

/// Upgraded version of make_label function with explanation of the channel color when hovering on it
pub fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) {
    make_label(ui, label, show, color).on_hover_text_at_pointer(
        if show {
            get_channel_type(color)
        } else {
            get_channel_type(Color32::WHITE)
        }
    );
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

    /// Technology (LTE or NR)
    technology: Technology,
}

impl LogicalChannels {
    /// Create a new LogicalChannels
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Handle the logic of the panel
    pub fn handle_logic(&mut self) {
        let canal = self.canal.as_str().replace("-NR", ""); // merge NR and LTE for now
        let canal_msg = self.canal_msg.as_str();
        // println!("canal: {}", canal);
        // println!("canal_msg: {}", canal_msg);
        match (canal.as_str(), canal_msg) {
            ("BCCH-BCH", "Master Information Block")  //4G
            | ("BCCH-BCH", "MIB") => {
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
            ("CCCH", "RRC connection request") //4G
            | ("CCCH", "RRC connection reestablishment request") //4G
            | ("CCCH", "RRC setup reestablishment request")
            | ("CCCH", "RRC setup request") => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_CCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("CCCH", "RRC connection setup") //4G
            | ("CCCH", "RRC connection reestablishment") //4G
            | ("CCCH", "RRC setup reestablishment") //5G
            | ("CCCH", "RRC setup") //5G
            => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_CCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            ("DCCH", "RRC connection setup complete") //4G
            | ("DCCH", "UL information transfer") 
            | ("DCCH", "Security mode complete") 
            | ("DCCH", "UE capability information") 
            | ("DCCH", "RRC connection reconfiguration complete") //4G
            | ("DCCH", "RRC connection reestablishment complete") //4G
            | ("DCCH", "RRC setup complete") //5G
            | ("DCCH", "RRC reconfiguration complete") //5G
            | ("DCCH", "RRC reestablishment complete") //5G
            => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::UL_DCCH,
                    transport: TransportChannelsEnum::UL_SCH,
                    physical: PhysicalChannelsEnum::PUSCH,
                });
            }
            ("DCCH", "DL information transfer") 
            | ("DCCH", "Security mode command") 
            | ("DCCH", "UE capability enquiry")
            | ("DCCH", "RRC connection reconfiguration") //4G
            | ("DCCH", "RRC connection release") //4G
            | ("DCCH", "RRC reconfiguration") //5G
            | ("DCCH", "RRC release") //5G
            => {
                self.state = Some(ChannelState {
                    logical: LogicalChannelsEnum::DL_DCCH,
                    transport: TransportChannelsEnum::DL_SCH,
                    physical: PhysicalChannelsEnum::PDSCH,
                });
            }
            _ => {
                // log::debug!("Unknown message : {}", self.canal_msg);
                self.state = None;
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
            print_on_grid(ui, "----");
            print_on_grid(ui, "----");
            print_on_grid(ui, "----");
            print_on_grid(ui, "Downlink");
            print_on_grid(ui, "----");
            print_on_grid(ui, "----");
            print_on_grid(ui, "----");
            print_on_grid(ui, "Techno: ");
            print_on_grid(ui, &self.technology.to_string());
            print_on_grid(ui, "");
            print_on_grid(ui, "----");
            print_on_grid(ui, "Uplink");
            print_on_grid(ui, "----");
            ui.end_row();

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
        });
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for LogicalChannels {
    fn on_event_added(&mut self, event: &Trace, _index: usize, context: &EventContext) {
        // Update technology from context metadata
        if self.technology != context.metadata.technology {
            self.technology = context.metadata.technology;
        }

        // Extract RRC info from the event
        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            self.canal = infos.canal.to_owned();
            self.canal_msg = infos.canal_msg.to_owned();
            self.handle_logic();
        }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates, update the displayed channels
        self.current_index = index;
        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            // log::debug!("Logical Channels: RRC message");
            self.canal = infos.canal.to_owned();
            self.canal_msg = infos.canal_msg.to_owned();
            self.handle_logic();
        } else {
            // Not an RRC message - clear the channel highlights
            // log::debug!("Logical Channels: Not an RRC message");
            self.state = None;
        }
    }

    fn on_events_cleared(&mut self) {
        log::debug!("Logical Channels: Clearing all state");
        self.canal.clear();
        self.canal_msg.clear();
        self.state = None;
        self.current_index = 0;
    }

    fn name(&self) -> &'static str {
        "Logical Channels"
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Logical Channels")
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        Ok(())
    }
}
