//! Logical Channels panel

use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::PanelView;
use crate::theme::{ChannelColors, ThemeColors};
use eframe::egui;
use egui::{Color32, TextFormat};
use tramex_tools::data::{AdditionalInfos, Trace};
use tramex_tools::errors::TramexError;
use tramex_tools::interface::parse_config::Technology;

/// Create a label with a background color (using Color32 directly)
pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) -> egui::Response {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let theme = ThemeColors::get(ui);

    // Use theme-aware text color - dark text on colored backgrounds for readability
    let text_color = if show { ChannelColors::TEXT_ON_COLOR } else { theme.text };

    let background = if show { color } else { Color32::TRANSPARENT };

    job.append(
        label,
        0.0,
        TextFormat {
            color: text_color,
            background,
            ..Default::default()
        },
    );
    ui.label(job)
}

/// Enumerate all types of logical channels in LTE & NR technologies
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
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

impl LogicalChannelsEnum {
    /// Get the color of the logical channel
    pub fn get_color(&self) -> Color32 {
        match self {
            LogicalChannelsEnum::PCCH => ChannelColors::BLUE,
            LogicalChannelsEnum::BCCH => ChannelColors::RED,
            LogicalChannelsEnum::DL_CCCH => ChannelColors::BLUE,
            LogicalChannelsEnum::DL_DCCH => ChannelColors::ORANGE,
            LogicalChannelsEnum::DL_DTCH => ChannelColors::GREEN,
            LogicalChannelsEnum::MCCH => ChannelColors::BLUE,
            LogicalChannelsEnum::MTCH => ChannelColors::GREEN,
            LogicalChannelsEnum::UL_CCCH => ChannelColors::BLUE,
            LogicalChannelsEnum::UL_DCCH => ChannelColors::ORANGE,
            LogicalChannelsEnum::UL_DTCH => ChannelColors::GREEN,
        }
    }
}

impl std::fmt::Display for LogicalChannelsEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            LogicalChannelsEnum::PCCH => "PCCH",
            LogicalChannelsEnum::BCCH => "BCCH",
            LogicalChannelsEnum::DL_CCCH => "CCCH",
            LogicalChannelsEnum::DL_DCCH => "DCCH",
            LogicalChannelsEnum::DL_DTCH => "DTCH",
            LogicalChannelsEnum::MCCH => "MCCH",
            LogicalChannelsEnum::MTCH => "MTCH",
            LogicalChannelsEnum::UL_CCCH => "CCCH",
            LogicalChannelsEnum::UL_DCCH => "DCCH",
            LogicalChannelsEnum::UL_DTCH => "DTCH",
        };
        write!(f, "{str}")
    }
}

/// Enumerate all types of transport channels in LTE & NR technologies
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
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

impl TransportChannelsEnum {
    /// Get the color of the transport channel
    pub fn get_color(&self) -> Color32 {
        match self {
            TransportChannelsEnum::PCH => ChannelColors::BLUE,
            TransportChannelsEnum::BCH => ChannelColors::RED,
            TransportChannelsEnum::DL_SCH => ChannelColors::GREEN,
            TransportChannelsEnum::MCH => ChannelColors::GREEN,
            TransportChannelsEnum::RACH => ChannelColors::BLUE,
            TransportChannelsEnum::UL_SCH => ChannelColors::GREEN,
        }
    }
}

impl std::fmt::Display for TransportChannelsEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            TransportChannelsEnum::PCH => "PCH",
            TransportChannelsEnum::BCH => "BCH",
            TransportChannelsEnum::DL_SCH => "DL-SCH",
            TransportChannelsEnum::MCH => "MCH",
            TransportChannelsEnum::RACH => "RACH",
            TransportChannelsEnum::UL_SCH => "UL-SCH",
        };
        write!(f, "{str}")
    }
}

/// Enumerate all types of physical channels in LTE & NR technologies
#[derive(PartialEq)]
#[allow(non_camel_case_types)]
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

impl PhysicalChannelsEnum {
    /// Get the color of the physical channel
    pub fn get_color(&self) -> Color32 {
        match self {
            PhysicalChannelsEnum::PDSCH => ChannelColors::GREEN,
            PhysicalChannelsEnum::PBCH => ChannelColors::RED,
            PhysicalChannelsEnum::PDCCH => ChannelColors::ORANGE,
            PhysicalChannelsEnum::PMCH => ChannelColors::GREEN,
            PhysicalChannelsEnum::PRACH => ChannelColors::BLUE,
            PhysicalChannelsEnum::PUSCH => ChannelColors::GREEN,
            PhysicalChannelsEnum::PUCCH => ChannelColors::ORANGE,
        }
    }
}

impl std::fmt::Display for PhysicalChannelsEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            PhysicalChannelsEnum::PDSCH => "PDSCH",
            PhysicalChannelsEnum::PBCH => "PBCH",
            PhysicalChannelsEnum::PDCCH => "PDCCH",
            PhysicalChannelsEnum::PMCH => "PMCH",
            PhysicalChannelsEnum::PRACH => "PRACH",
            PhysicalChannelsEnum::PUSCH => "PUSCH",
            PhysicalChannelsEnum::PUCCH => "PUCCH",
        };
        write!(f, "{str}")
    }
}

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
    make_label(ui, label, show, color).on_hover_text_at_pointer(if show {
        get_channel_type(color)
    } else {
        get_channel_type(Color32::WHITE)
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
            ("BCCH", "SIB1") | ("BCCH", "SIB2") | ("BCCH", "SIB3") | ("BCCH", "SIB") => {
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
    fn name(&self) -> &'static str {
        "Logical Channels"
    }

    fn on_metadata_changed(&mut self, metadata: &tramex_tools::interface::parse_config::FileMetadata) {
        // Update technology from context metadata
        if self.technology != metadata.technology {
            self.technology = metadata.technology;
        }
    }

    fn on_event_added(&mut self, event: &Trace, _index: usize, _context: &EventContext) {
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
