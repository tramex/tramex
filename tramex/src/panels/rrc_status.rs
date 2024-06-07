//! Panel to display the RRC status
use eframe::egui::Color32;
use tramex_tools::data::Data;
use tramex_tools::data::Trace;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::types::Direction;

use super::functions_panels::make_arrow;
use super::functions_panels::make_label_equal;
use super::functions_panels::ArrowColor;
use super::functions_panels::ArrowDirection;
use super::functions_panels::CustomLabelColor;

/// Panel to display the RRC status
#[derive(Debug, Default)]
pub struct LinkPanel {
    /// current trace
    current_trace: Option<Trace>,

    /// direction
    direction: Option<Direction>,

    /// Arrow font
    font_id: egui::FontId,
}

impl LinkPanel {
    /// Create a new instance of the LinkPanel
    pub fn new() -> Self {
        Self {
            current_trace: None,
            font_id: egui::FontId::monospace(60.0),
            direction: None,
        }
    }

    /// Display the control of the link
    pub fn ui_control(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| match self.direction {
            Some(Direction::UL) => {
                ui.colored_label(egui::Color32::RED, "Link Control");
            }
            Some(Direction::DL) => {
                ui.colored_label(egui::Color32::BLUE, "Link Control");
            }
            _ => {
                ui.label("Link Control");
            }
        });
    }

    /// Print on the grid
    pub fn print_on_grid(&self, ui: &mut egui::Ui, label: &str) {
        ui.vertical_centered(|ui| {
            ui.label(label);
        });
    }

    /// Make a colored label

    /// Display the connection state of the LTE
    pub fn ui_con(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui|{
            let etat = match self.direction {
                Some(Direction::UL) => "PCCH",
                Some(Direction::DL) => "BCCH",
                _ => "Unknown",
            };
    
            let available_width = ui.available_width();
            let max_label_width = 100.0; // Maximum label width
            let num_labels = 2; // Number of labels horizontally
            let total_max_label_width = max_label_width * num_labels as f32;
            let label_width = (available_width / num_labels as f32).min(max_label_width);
            let space_between_labels = (available_width - total_max_label_width) / (num_labels + 1) as f32;
    
            let color = match self.direction {
                Some(Direction::UL) => egui::Color32::RED,
                Some(Direction::DL) => egui::Color32::BLUE,
                _ => egui::Color32::BLACK,
            };
    
            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_equal(ui, "PCCH", etat, CustomLabelColor::Red);
                ui.add_space(space_between_labels);
                self.print_on_grid(ui, "|");
                ui.add_space(space_between_labels);
                make_label_equal(ui, "BCCH", etat, CustomLabelColor::Red);
            });
    
            ui.horizontal(|ui| {
                ui.add_space(space_between_labels);
                make_label_equal(ui, "PCH", etat, CustomLabelColor::Red);
                ui.add_space(space_between_labels);
                self.print_on_grid(ui, "|");
                ui.add_space(space_between_labels);
                make_label_equal(ui, "BCH", etat, CustomLabelColor::Red);
            });
        });
    }
    

    /// Display the idle state of the LTE
    pub fn ui_idle_lte(&self, ui: &mut egui::Ui) {
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
        let available_width = ui.available_width();
        let arrow_width = 60.0; // Assuming the arrow's width is 60.0 units, adjust as needed
        let num_arrows = 2; // Number of arrows horizontally
        let total_arrow_width = arrow_width * num_arrows as f32; // Cast num_arrows to f32
        let space_between_arrows = (available_width - total_arrow_width) / (num_arrows + 1) as f32;
    
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
    }
    

    /// Display the content of the link
    pub fn ui_content_level2(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let arrow_width = 60.0; // Assuming the arrow's width is 60.0 units, adjust as needed
        let num_arrows = 4; // Number of arrows horizontally
        let total_arrow_width = arrow_width * num_arrows as f32; // Cast num_arrows to f32
        let space_between_arrows = (available_width - total_arrow_width) / (num_arrows + 1) as f32;
    
        ui.horizontal(|ui| {
            ui.add_space(space_between_arrows);
            match self.direction {
                Some(Direction::UL) => {
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Green, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Green, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                }
                Some(Direction::DL) => {
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
                }
                _ => {
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                    ui.add_space(space_between_arrows);
                    make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                }
            }
            ui.add_space(space_between_arrows);
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
        if let Some(trace) = data.get_current_trace() {
            self.current_trace = Some(trace.clone());
            self.direction = Some(trace.trace_type.direction.clone());
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
