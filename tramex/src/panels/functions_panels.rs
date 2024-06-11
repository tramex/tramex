//! Panels functions

use egui::{Color32, TextFormat};

/// Custom label color
#[derive(Clone)]
pub enum CustomLabelColor {
    /// Red color
    Red,

    /// Blue color
    Blue,

    /// Orange color
    Orange,

    /// Green color
    Green,

    /// White color
    White,
}

impl CustomLabelColor {
    /// Get the type of channel
    pub fn get_type_channel(&self) -> &'static str {
        match self {
            CustomLabelColor::Red => "Broadcast channel",
            CustomLabelColor::Blue => "Common channel",
            CustomLabelColor::Green => "Traffic channel",
            CustomLabelColor::Orange => "Dedicated channel",
            CustomLabelColor::White => "This channel is currently unused",
        }
    }
}

/// Print a label on the grid
pub fn make_label_equal(ui: &mut egui::Ui, label: &str, state: &str, color: CustomLabelColor) {
    make_label(ui, label, label == state, color);
}

/// Create a label with a background color
pub fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) -> egui::Response {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let (default_color, _strong_color) = (Color32::BLACK, Color32::BLACK);
    let background = if show {
        match color {
            CustomLabelColor::Red => Color32::from_rgb(255, 84, 84),
            CustomLabelColor::Blue => Color32::from_rgb(68, 143, 255),
            CustomLabelColor::Orange => Color32::from_rgb(255, 181, 68),
            CustomLabelColor::Green => Color32::from_rgb(90, 235, 100),
            CustomLabelColor::White => Color32::from_rgb(255, 255, 255),
        }
    } else {
        Color32::from_rgb(255, 255, 255)
    };

    job.append(
        label,
        0.0,
        TextFormat {
            color: default_color,
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
#[derive(Debug)]
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
    let current_color = match color {
        ArrowColor::Green => Color32::from_rgb(110, 255, 110),
        ArrowColor::Blue => Color32::from_rgb(68, 143, 255),
        ArrowColor::Black => Color32::from_rgb(0, 0, 0),
    };

    ui.label(egui::RichText::new(content).color(current_color).font(font_id.clone()));
}

/// Enumerate all types of logical channels in LTE technology
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
    pub fn get_color(&self) -> CustomLabelColor {
        match self {
            LogicalChannelsEnum::PCCH => CustomLabelColor::Blue,
            LogicalChannelsEnum::BCCH => CustomLabelColor::Red,
            LogicalChannelsEnum::DL_CCCH => CustomLabelColor::Blue,
            LogicalChannelsEnum::DL_DCCH => CustomLabelColor::Orange,
            LogicalChannelsEnum::DL_DTCH => CustomLabelColor::Green,
            LogicalChannelsEnum::MCCH => CustomLabelColor::Blue,
            LogicalChannelsEnum::MTCH => CustomLabelColor::Green,
            LogicalChannelsEnum::UL_CCCH => CustomLabelColor::Blue,
            LogicalChannelsEnum::UL_DCCH => CustomLabelColor::Orange,
            LogicalChannelsEnum::UL_DTCH => CustomLabelColor::Green,
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
        write!(f, "{}", str)
    }
}

/// Enumerate all types of transport channels in LTE technology
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
    /// Get the color of the logical channel
    pub fn get_color(&self) -> CustomLabelColor {
        match self {
            TransportChannelsEnum::PCH => CustomLabelColor::Blue,
            TransportChannelsEnum::BCH => CustomLabelColor::Red,
            TransportChannelsEnum::DL_SCH => CustomLabelColor::Green,
            TransportChannelsEnum::MCH => CustomLabelColor::Green,
            TransportChannelsEnum::RACH => CustomLabelColor::Blue,
            TransportChannelsEnum::UL_SCH => CustomLabelColor::Green,
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
        write!(f, "{}", str)
    }
}

/// Enumerate all types of physical channels in LTE technology
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
    /// Get the color of the logical channel
    pub fn get_color(&self) -> CustomLabelColor {
        match self {
            PhysicalChannelsEnum::PDSCH => CustomLabelColor::Green,
            PhysicalChannelsEnum::PBCH => CustomLabelColor::Red,
            PhysicalChannelsEnum::PDCCH => CustomLabelColor::Orange,
            PhysicalChannelsEnum::PMCH => CustomLabelColor::Green,
            PhysicalChannelsEnum::PRACH => CustomLabelColor::Blue,
            PhysicalChannelsEnum::PUSCH => CustomLabelColor::Green,
            PhysicalChannelsEnum::PUCCH => CustomLabelColor::Orange,
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
        write!(f, "{}", str)
    }
}
