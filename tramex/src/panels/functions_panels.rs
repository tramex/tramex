//! Panels functions

use egui::{Color32, TextFormat};
use crate::theme::{ThemeColors, ChannelColors, ArrowColors};

/// Create a label with a background color (using Color32 directly)
pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) -> egui::Response {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let theme = ThemeColors::get(ui);
    
    // Use theme-aware text color - dark text on colored backgrounds for readability
    let text_color = if show {
        ChannelColors::TEXT_ON_COLOR
    } else {
        theme.text
    };
    
    let background = if show {
        color
    } else {
        Color32::TRANSPARENT
    };

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

/// Arrow direction
#[derive(Debug)]
pub enum ArrowDirection {
    /// Up arrow
    Up,

    /// Down arrow
    Down,
}

/// Arrow color
#[derive(Debug, Clone)]
pub enum ArrowColor {
    /// Green arrow
    Green,

    /// Blue arrow
    Blue,

    /// Black arrow
    Black,
}

/// Create an arrow
pub fn make_arrow(ui: &mut egui::Ui, direction: ArrowDirection, color: ArrowColor, font_id: &egui::FontId) {
    // ↑↓
    // ⇑⇓
    // ⇡⇣ chosen
    // ⮉⮋
    // ⬆⬇
    // ⇧⇩
    let content = match direction {
        ArrowDirection::Down => "⇣",
        ArrowDirection::Up => "⇡",
    };
    let theme = ThemeColors::get(ui);
    let current_color = match color {
        ArrowColor::Green => ArrowColors::ACTIVE,
        ArrowColor::Blue => ArrowColors::HIGHLIGHT,
        ArrowColor::Black => theme.text,
    };

    ui.label(egui::RichText::new(content).color(current_color).font(font_id.clone()));
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
