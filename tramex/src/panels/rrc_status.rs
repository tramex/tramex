//! Panel to display the RRC status
use super::functions_panels::make_arrow;
use super::functions_panels::make_label;
use super::functions_panels::ArrowColor;
use super::functions_panels::ArrowDirection;
use super::functions_panels::CustomLabelColor;
use tramex_tools::data::AdditionalInfos;
use tramex_tools::data::Data;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::types::Direction;

/// Make a label with hover effect
fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) {
    make_label(ui, label, show, color);
}

/// Panel to display the RRC status
#[derive(Default)]
pub struct LinkPanel {
    /// Canal
    canal: Option<String>,

    /// Canal message
    canal_msg: Option<String>,

    /// Direction
    direction: Option<Direction>,

    /// Current index
    current_index: usize,

    /// Font id
    font_id: egui::FontId,

    /// Is connected
    is_connected: bool,
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
            is_connected: false,
        }
    }

    /// Display the control of the link
    pub fn ui_control(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            make_label_hover(ui, "CONNECTED", self.is_connected, CustomLabelColor::Green);
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
                None => "",
            };

            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_hover(ui, "Ura PCH", canal == "Ura PCH", CustomLabelColor::Blue);
                ui.add_space(space_between_labels);
                ui.label(" |  ");
                ui.add_space(space_between_labels);
                make_label_hover(ui, "Cell PCH", canal == "Cell PCH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
            });

            let max_label_width = 3.0 * 18.0; // Maximum label width
            let num_labels = 3; // Number of labels horizontally
            let total_max_label_width = max_label_width * num_labels as f32;
            let space_between_labels = ((available_width - total_max_label_width) / (num_labels + 1) as f32).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_hover(ui, "Cell DCH", canal == "Cell DCH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
                ui.label(" | ");
                ui.add_space(space_between_labels);
                make_label_hover(ui, "Cell FACH", canal == "Cell FACH", CustomLabelColor::Red);
                ui.add_space(space_between_labels);
            });
        });
    }

    /// Display the idle state of the LTE
    pub fn ui_idle_lte(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            make_label_hover(ui, "IDLE", !self.is_connected, CustomLabelColor::Red);
        });
    }

    /// Display the LTE state
    pub fn ui_lte(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.colored_label(egui::Color32::BLUE, "LTE");
        });
    }

    /// Display the idle state of the UMTS
    pub fn ui_idle_umts(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.colored_label(egui::Color32::BLACK, "IDLE");
        });
    }

    /// Display the UMTS state
    pub fn ui_umts(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.colored_label(egui::Color32::BLUE, "UMTS");
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
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                    Some(Direction::DL) => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
                    }
                    _ => {
                        make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    }
                };
                ui.add_space(space_between_arrows);
                match self.direction {
                    Some(Direction::UL) => {
                        make_arrow(ui, ArrowDirection::Up, ArrowColor::Green, &self.font_id);
                    }
                    Some(Direction::DL) => {
                        make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                    }
                    _ => {
                        make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
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
                    Some(Direction::UL) => (ArrowColor::Black, ArrowColor::Green),
                    Some(Direction::DL) => (ArrowColor::Green, ArrowColor::Black),
                    _ => (ArrowColor::Black, ArrowColor::Black),
                };
                make_arrow(ui, ArrowDirection::Down, color1.clone(), &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Down, color1, &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Up, color2.clone(), &self.font_id);
                ui.add_space(space_between_arrows);
                make_arrow(ui, ArrowDirection::Up, color2, &self.font_id);
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
                        if self.current_index < data.current_index {
                            match (self.is_connected, self.canal_msg.as_deref()) {
                                (true, Some("RRC connection release")) => self.is_connected = false,
                                (false, Some("RRC connection setup complete")) => self.is_connected = true,
                                _ => {}
                            }
                        } else {
                            match (self.is_connected, self.canal_msg.as_deref()) {
                                (true, Some("RRC connection setup complete")) => self.is_connected = false,
                                (false, Some("RRC connection release")) => self.is_connected = true,
                                _ => {}
                            }
                        }

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
