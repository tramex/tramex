//! Panel to display the RRC status
use tramex_tools::data::Data;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::types::Direction;
use tramex_tools::data::AdditionalInfos;
use super::functions_panels::make_arrow;
use super::functions_panels::make_label;
use super::functions_panels::ArrowColor;
use super::functions_panels::ArrowDirection;
use super::functions_panels::CustomLabelColor;

fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) {
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

/// Struct that contains the three logical channels associated to the current message
#[derive(PartialEq)]
struct LogicalChannelState {
    /// Logical channel of current message
    logical: LogicalChannelsEnum,

    /// Transport channel of current message
    transport: TransportChannelsEnum,
}

/// Panel to display the RRC status
#[derive(Default)]
pub struct LinkPanel {
    canal: Option<String>,
    canal_msg: Option<String>,
    direction: Option<Direction>,
    current_index: usize,
    font_id: egui::FontId
}

impl LinkPanel {
    /// Create a new instance of the LinkPanel
    pub fn new() -> Self {
        Self {
            font_id: egui::FontId::monospace(60.0),
            direction: None,
            canal: None,
            canal_msg: None,
            current_index: 0,
        }
    }

    /// Display the control of the link
pub fn ui_control(&self, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        let canal_msg = self.canal_msg.as_deref().unwrap_or("");
        let show_red = canal_msg == "RRC connection setup";
        let show_white = canal_msg == "RRC connection release";

        let label_color = if show_red {
            egui::Color32::RED
        } else if show_white {
            egui::Color32::WHITE
        } else {
            ui.style().visuals.text_color()
        };

        match self.direction {
            Some(Direction::UL) => {
                ui.colored_label(label_color, "CONNECTED");
            }
            Some(Direction::DL) => {
                ui.colored_label(label_color, "CONNECTED");
                if show_white {
                    make_label_hover(ui, "CONNECTED", show_white, CustomLabelColor::White);
                }
            }
            _ => {
                ui.label("CONNECTED");
            }
        }
    });
}


            
    /// Display the connection state of the LTE
    pub fn ui_con(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            

            let available_width = ui.available_width();
            let max_label_width = 4.0 * 18.0; // Maximum label width
            let num_labels = 3; // Number of labels horizontally
            let total_max_label_width = max_label_width * num_labels as f32;
            let space_between_labels = ((available_width - total_max_label_width) / (num_labels + 1) as f32).max(0.0);

            let canal = match &self.canal {
                Some(r) => r,
                None => ""
            };

            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_hover(ui, "PCCH", canal == "PCCH", CustomLabelColor::Blue);
                ui.add_space(space_between_labels);
                ui.label(" |  ");
                ui.add_space(space_between_labels);
                make_label_hover(ui, "BCCH", canal == "BCCH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
            });

            let max_label_width = 3.0 * 18.0; // Maximum label width
            let num_labels = 3; // Number of labels horizontally
            let total_max_label_width = max_label_width * num_labels as f32;
            let space_between_labels = ((available_width - total_max_label_width) / (num_labels + 1) as f32).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_hover(ui, "PCH", canal == "PCH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
                ui.label(" | ");
                ui.add_space(space_between_labels);
                make_label_hover(ui, "BCH", canal == "BCH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
            });
        });
    }

    /// Display the idle state of the LTE
    pub fn ui_idle_lte(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            
            
            let canal_msg = match &self.canal_msg {
                Some(r) => r,
                None => ""
            };
            
            match self.direction {
            Some(Direction::UL) => {
                make_label_hover(ui, "IDLE", canal_msg == "RRC connection release", CustomLabelColor::Red);
            }
            Some(Direction::DL) => {
                make_label_hover(ui, "IDLE", canal_msg == "RRC connection release", CustomLabelColor::Red); 
            }
            _ => {
                ui.label("IDLE");
            }
        }
        });
    }

    /// Display the LTE state
    pub fn ui_lte(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| match self.direction {
            Some(Direction::UL) => {
                ui.colored_label(egui::Color32::GREEN, "LTE");
            }
            Some(Direction::DL) => {
                ui.colored_label(egui::Color32::BLACK, "LTE");
            }
            _ => {
                ui.label("LTE");
            }
        });
    }

    /// Display the idle state of the UMTS
    pub fn ui_idle_umts(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| match self.direction {
            Some(Direction::UL) => {
                ui.colored_label(egui::Color32::RED, "IDLE");
            }
            Some(Direction::DL) => {
                ui.colored_label(egui::Color32::BLACK, "IDLE");
            }
            _ => {
                ui.label("IDLE");
            }
        });
    }

    /// Display the UMTS state
    pub fn ui_umts(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| match self.direction {
            Some(Direction::UL) => {
                ui.colored_label(egui::Color32::RED, "UMTS");
            }
            Some(Direction::DL) => {
                ui.colored_label(egui::Color32::BLACK, "UMTS");
            }
            _ => {
                ui.label("UMTS");
            }
        });
    }

    /// Display the content of the link
    pub fn ui_content(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            let available_width = ui.available_width();
            let arrow_width = 60.0 - 15.0; // Assuming the arrow's width is 60.0 units, adjust as needed
            let num_arrows = 2; // Number of arrows horizontally
            let total_arrow_width = arrow_width * num_arrows as f32; // Cast num_arrows to f32
            let space_between_arrows = ((available_width - total_arrow_width) / (num_arrows + 1) as f32).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(space_between_arrows);
                match self.direction {
                    Some(Direction::UL) => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Blue, &self.font_id);
                    }
                    Some(Direction::DL) => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                    _ => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                };
                ui.add_space(space_between_arrows);
                match self.direction {
                    Some(Direction::UL) => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                    Some(Direction::DL) => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
                    }
                    _ => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                }
                ui.add_space(space_between_arrows);
            });
        });
    }

    /// Display the content of the link
    pub fn ui_content_level2(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            let available_width = ui.available_width();
            let arrow_width = 60.0 - 15.0; // Assuming the arrow's width is 60.0 units, adjust as needed
            let num_arrows = 4; // Number of arrows horizontally
            let total_arrow_width = arrow_width * num_arrows as f32; // Cast num_arrows to f32
            let space_between_arrows = ((available_width - total_arrow_width) / (num_arrows + 1) as f32).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(space_between_arrows);
                let (color1, color2) = match self.direction {
                    Some(Direction::UL) => (ArrowColor::Green, ArrowColor::Black),
                    Some(Direction::DL) => (ArrowColor::Black, ArrowColor::Green),
                    _ => (ArrowColor::Black, ArrowColor::Black),
                };
                make_arrow(ui, ArrowDirection::Up, color1.clone(), &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Up, color1, &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Down, color2.clone(), &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Down, color2, &self.font_id);
                ui.add_space(space_between_arrows);
            });
        });
    }
}

impl super::PanelController for LinkPanel {
    fn name(&self) -> &'static str {
        "RRC Status"
    }

    fn window_title(&self) -> &'static str {
        "RRC Status"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError> {
        if data.is_different_index(self.current_index) {
            if let Some(one_trace) = data.get_current_trace() {
                match &one_trace.additional_infos {
                    AdditionalInfos::RRCInfos(infos) => {
                        self.canal = Some(infos.canal.to_owned());
                        self.canal_msg = Some(infos.canal_msg.to_owned());
                        self.direction = Some(infos.direction.clone());
                    }
                }
            }
            self.current_index = data.current_index;
        }
        egui::Window::new(self.window_title())
            .default_width(160.0)
            .default_height(160.0)
            .open(open)
            .resizable([true, true])
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui)
            });
        Ok(())
    }
}

impl super::PanelView for LinkPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_control(ui);
        ui.separator();
        self.ui_content(ui);
        ui.separator();
        self.ui_idle_lte(ui);
        ui.separator();
        self.ui_lte(ui);
        ui.separator();
        self.ui_con(ui);
        ui.separator();
        self.ui_content_level2(ui);
        ui.separator();
        self.ui_idle_umts(ui);
        ui.separator();
        self.ui_umts(ui);
    }
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