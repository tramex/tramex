//! About panel
use eframe::egui;
use tramex_tools::errors::TramexError;

#[derive(Default)]
/// About panel
pub struct AboutPanel {}

impl super::PanelController for AboutPanel {
    fn name(&self) -> &'static str {
        "About"
    }
    fn window_title(&self) -> &'static str {
        "About"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
        Ok(())
    }
}

impl super::PanelView for AboutPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("egui");

        ui.add_space(12.0); // ui.separator();
        ui.heading("Links");

        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to("github", "https://github.com/tramex/tramex");
        });

        ui.add_space(12.0);
    }
}
