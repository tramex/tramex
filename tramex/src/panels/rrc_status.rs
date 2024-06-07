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
        let etat = match self.direction {
            Some(Direction::UL) => "PCCH",
            Some(Direction::DL) => "BCCH",
            _ => "Unknown",
        };

        let dark_mode = ui.visuals().dark_mode;
        let faded_color = ui.visuals().window_fill();
        let _faded_color = |color: Color32| -> Color32 {
            use egui::Rgba;
            let t = if dark_mode { 0.95 } else { 0.8 };
            egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
        };
        //let etat = "PCCH";

        egui::Grid::new("some_unique_id").max_col_width(50.0).show(ui, |ui| {
            ui.add_space(20.0);
            make_label_equal(ui, "PCCH", etat, CustomLabelColor::Red);
            self.print_on_grid(ui, "|");
            make_label_equal(ui, "BCCH", etat, CustomLabelColor::Red);
            ui.end_row();

            ui.add_space(20.0);
            make_label_equal(ui, "PCH", etat, CustomLabelColor::Red);
            self.print_on_grid(ui, "|");
            make_label_equal(ui, "BCH", etat, CustomLabelColor::Red);
            ui.end_row();
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
        ui.horizontal_wrapped(|ui| {
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
            ui.min_rect();
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
        });
    }

    /// Display the content of the link
    pub fn ui_content_level2(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| match self.direction {
            Some(Direction::UL) => {
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Green, &self.font_id);
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Green, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
            }
            Some(Direction::DL) => {
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Green, &self.font_id);
            }
            _ => {
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Up, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
                make_arrow(ui, ArrowDirection::Down, ArrowColor::Black, &self.font_id);
            }
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
