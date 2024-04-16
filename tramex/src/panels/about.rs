use eframe::egui;

#[derive(Default)]
pub struct AboutPanel {}

impl super::PanelController for AboutPanel {
    fn name(&self) -> &'static str {
        "About"
    }
    fn window_title(&self) -> &'static str {
        "About"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
    }
}

impl super::PanelView for AboutPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        use egui::special_emojis::{OS_APPLE, OS_LINUX, OS_WINDOWS};

        ui.heading("egui");
        ui.label(format!(
            "egui is an immediate mode GUI library written in Rust. egui runs both on the web and natively on {}{}{}. \
            On the web it is compiled to WebAssembly and rendered with WebGL.{}",
            OS_APPLE, OS_LINUX, OS_WINDOWS,
            if cfg!(target_arch = "wasm32") {
                " Everything you see is rendered as textured triangles. There is no DOM, HTML, JS or CSS. Just Rust."
            } else {""}
        ));

        ui.add_space(12.0); // ui.separator();
        ui.heading("Links");

        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(
                "notes.rezel.net",
                "https://notes.rezel.net/22PBCZhXTvGsG5ipptTvwQ",
            );
        });

        ui.add_space(12.0);
    }
}
