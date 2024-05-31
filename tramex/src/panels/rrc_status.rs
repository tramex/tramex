use eframe::egui::Color32;
use eframe::egui::{self, TextFormat};
use std::cell::RefCell;
use std::primitive::f32;
use std::rc::Rc;
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;
use tramex_tools::websocket::types::Direction;

pub struct LinkPannel {
    mtrace: Rc<RefCell<Connector>>,
}

impl LinkPannel {
    pub fn new(ref_data: Rc<RefCell<Connector>>) -> Self {
        Self { mtrace: ref_data }
    }
}
impl LinkPannel {
    pub fn ui_control(&self, ui: &mut egui::Ui, direction: &Direction) {
        ui.vertical_centered_justified(|ui| match direction {
            Direction::UL => {
                ui.colored_label(egui::Color32::RED, "Link Control");
            }
            Direction::DL => {
                ui.colored_label(egui::Color32::BLUE, "Link Control");
            }
            _ => {
                ui.label("Link Control");
            }
        });
    }

    pub fn print_on_grid(&self, ui: &mut egui::Ui, label: &str) {
        ui.vertical_centered(|ui| {
            ui.label(label);
        });
    }

    pub fn make_label(&self, ui: &mut egui::Ui, label: &str, state: &str, color: &str) {
        use egui::text::LayoutJob;
        let mut job = LayoutJob::default();
        let (default_color, _strong_color) = (Color32::BLACK, Color32::BLACK);
        let background = if label == state {
            match color {
                "red" => Color32::from_rgb(255, 84, 84),
                "blue" => Color32::from_rgb(68, 143, 255),
                "orange" => Color32::from_rgb(255, 181, 68),
                "green" => Color32::from_rgb(90, 235, 100),
                _ => Color32::from_rgb(90, 235, 100),
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
        ui.vertical_centered(|ui| {
            ui.label(job);
        });
    }

    pub fn ui_con(&self, ui: &mut egui::Ui, direction: &Direction) {
        let etat = match direction {
            Direction::UL => "PCCH",
            Direction::DL => "BCCH",
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
            self.make_label(ui, "PCCH", &etat, "red");
            self.print_on_grid(ui, "|");
            self.make_label(ui, "BCCH", &etat, "red");
            ui.end_row();

            ui.add_space(20.0);
            self.make_label(ui, "PCH", &etat, "red");
            self.print_on_grid(ui, "|");
            self.make_label(ui, "BCH", &etat, "red");
            ui.end_row();
        });
    }

    pub fn ui_idle_lte(&self, ui: &mut egui::Ui, direction: &Direction) {
        ui.vertical_centered_justified(|ui| match direction {
            Direction::UL => {
                ui.colored_label(egui::Color32::RED, "IDLE");
            }

            Direction::DL => {
                ui.colored_label(egui::Color32::BLACK, "IDLE");
            }

            _ => {
                ui.label("IDLE");
            }
        });
    }

    pub fn ui_lte(&self, ui: &mut egui::Ui, direction: &Direction) {
        ui.vertical_centered_justified(|ui| match direction {
            Direction::UL => {
                ui.colored_label(egui::Color32::GREEN, "LTE");
            }

            Direction::DL => {
                ui.colored_label(egui::Color32::BLACK, "LTE");
            }

            _ => {
                ui.label("LTE");
            }
        });
    }

    pub fn ui_idle_umts(&self, ui: &mut egui::Ui, direction: &Direction) {
        ui.vertical_centered_justified(|ui| match direction {
            Direction::UL => {
                ui.colored_label(egui::Color32::RED, "IDLE");
            }

            Direction::DL => {
                ui.colored_label(egui::Color32::BLACK, "IDLE");
            }

            _ => {
                ui.label("IDLE");
            }
        });
    }

    pub fn ui_umts(&self, ui: &mut egui::Ui, direction: &Direction) {
        ui.vertical_centered_justified(|ui| match direction {
            Direction::UL => {
                ui.colored_label(egui::Color32::RED, "UMTS");
            }

            Direction::DL => {
                ui.colored_label(egui::Color32::BLACK, "UMTS");
            }

            _ => {
                ui.label("UMTS");
            }
        });
    }

    pub fn ui_content(&self, ui: &mut egui::Ui, direction: &Direction) {
        const SPACE_RIGHT: f32 = 100.0;
        const SPACE_LEFT: f32 = 8.0;
        let size = egui::Vec2::new(50.0, 45.0);

        let upblack = egui::Image::new(egui::include_image!("../../assets/up.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);
        let downblack = egui::Image::new(egui::include_image!("../../assets/down.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);
        let downgreen = egui::Image::new(egui::include_image!("../../assets/down-green.png"))
            .max_size(size)
            .maintain_aspect_ratio(true);
        let upblue = egui::Image::new(egui::include_image!("../../assets/up-green.png"))
            .max_size(size)
            .maintain_aspect_ratio(true);

        //ui.vertical(|ui| match self.mtrace.direction.as_str() {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| match direction {
            Direction::UL => {
                ui.add_space(SPACE_LEFT);
                ui.add(upblue);
                ui.add_space(SPACE_RIGHT);
                ui.add(downblack);
            }
            Direction::DL => {
                ui.add_space(SPACE_LEFT);
                ui.add(upblack);
                ui.add_space(SPACE_RIGHT);
                ui.add(downgreen);
            }
            _ => {
                ui.add_space(SPACE_LEFT);
                ui.add(upblack);
                ui.add_space(SPACE_RIGHT);
                ui.add(downblack);
            }
        });
    }

    pub fn ui_content_level2(&self, ui: &mut egui::Ui, direction: &Direction) {
        const SPACE_RIGHT: f32 = 10.0;
        const SPACE_LEFT: f32 = 2.0;
        let size = egui::Vec2::new(50.0, 45.0);

        let upblack = egui::Image::new(egui::include_image!("../../assets/up.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);
        let downblack = egui::Image::new(egui::include_image!("../../assets/down.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);
        let downgreen = egui::Image::new(egui::include_image!("../../assets/down-green.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);
        let upgreen = egui::Image::new(egui::include_image!("../../assets/up-green.png"))
            .max_size(size)
            .fit_to_fraction(size)
            .maintain_aspect_ratio(true);

        //ui.vertical(|ui| match self.mtrace.direction.as_str() {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| match direction {
            Direction::UL => {
                ui.add_space(SPACE_LEFT);
                ui.add(upgreen.clone());
                ui.add(upgreen);
                ui.add_space(SPACE_RIGHT);
                ui.add(downblack.clone());
                ui.add(downblack);
            }
            Direction::DL => {
                ui.add_space(SPACE_LEFT);
                ui.add(upblack.clone());
                ui.add(upblack);
                ui.add_space(SPACE_RIGHT);
                ui.add(downgreen.clone());
                ui.add(downgreen);
            }
            _ => {
                ui.add_space(SPACE_LEFT);
                ui.add(upblack.clone());
                ui.add(upblack);
                ui.add_space(SPACE_RIGHT);
                ui.add(downblack.clone());
                ui.add(downblack);
            }
        });
    }
}

impl super::PanelController for LinkPannel {
    fn name(&self) -> &'static str {
        "RRC Status"
    }

    fn window_title(&self) -> &'static str {
        "RRC Status"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        let size = egui::Vec2::new(215.0, 200.0);
        egui::Window::new(self.name()).open(open).fixed_size(size).show(ctx, |ui| {
            use super::PanelView as _;
            self.ui(ui)
        });
        Ok(())
    }
}

impl super::PanelView for LinkPannel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let binding = self.mtrace.borrow();
        let curr_trace = binding.data.get_current_trace();
        if let Some(trace) = curr_trace {
            let direction = &trace.trace_type.direction;
            self.ui_control(ui, direction);
            ui.separator();
            self.ui_content(ui, direction);
            ui.separator();
            self.ui_idle_lte(ui, direction);
            ui.separator();
            self.ui_lte(ui, direction);
            ui.separator();
            self.ui_con(ui, direction);
            ui.separator();
            self.ui_content_level2(ui, direction);
            ui.separator();
            self.ui_idle_umts(ui, direction);
            ui.separator();
            self.ui_umts(ui, direction);
        }
    }
}
