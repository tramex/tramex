use eframe::egui::{self};
use egui::special_emojis::{OS_APPLE, OS_LINUX, OS_WINDOWS};
use tramex_tools::errors::TramexError;

use crate::frontend::FrontEnd;
use crate::make_hyperlink;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TramexApp {
    /// Frontend
    pub frontend: FrontEnd,
    #[serde(skip)]
    error_panel: Option<TramexError>,
    show_about: bool,
}

impl TramexApp {
    /// Load the app state from the given storage.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self {
            ..Default::default()
        }
    }
    fn menu_bar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);
        ui.separator();
        ui.menu_button("Menu", |ui| {
            if ui.button("Organize windows").clicked() {
                ui.ctx().memory_mut(|mem| mem.reset_areas());
            }
            if ui.button("About").clicked() {
                self.show_about = !self.show_about;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    _ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
        ui.menu_button("About", |ui| {
            make_hyperlink(
                ui,
                "User documentation",
                "https://tramex.github.io/tramex/docs/",
                true,
            );
            make_hyperlink(
                ui,
                "tramex types",
                "https://tramex.github.io/tramex/crates/tramex/",
                true,
            );
            make_hyperlink(
                ui,
                "tramex-tools types",
                "https://tramex.github.io/tramex/crates/tramex_tools/",
                true,
            );
            make_hyperlink(
                ui,
                "tramex repository",
                "https://github.com/tramex/tramex",
                true,
            );
        });
    }

    fn ui_error_panel(&mut self, ctx: &egui::Context) {
        if let Some(error_item) = &self.error_panel {
            let mut error_panel_open = true;
            egui::Window::new("Errors")
                .default_width(320.0)
                .default_height(480.0)
                .open(&mut error_panel_open)
                .resizable([true, false])
                .show(ctx, |ui| {
                    ui.label(format!("Error code: {}", error_item.get_code()));
                    if error_item.is_recoverable() {
                        ui.label("Recoverable error !");
                    }
                    ui.colored_label(egui::Color32::RED, &error_item.message);
                    if ui.button("Copy error").clicked() {
                        ui.output_mut(|o| {
                            o.copied_text =
                                format!("{}\n{}", &error_item.get_code(), &error_item.message,)
                        });
                    };
                    make_hyperlink(
                        ui,
                        "Report the issue",
                        "https://github.com/tramex/tramex/issues/new",
                        true,
                    );
                });
            if error_item.is_recoverable() && !error_panel_open {
                log::debug!("Closing error windows");
                self.error_panel = None;
            }
        }
    }

    fn ui_about_windows(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable([true, true])
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(format!("Name: {}", env!("CARGO_PKG_NAME")));
                    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                    ui.label(format!("Description: {}", env!("CARGO_PKG_DESCRIPTION")));
                    ui.label(format!("License: {}", env!("CARGO_PKG_LICENSE")));
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        ui.label("Repository: ");
                        make_hyperlink(
                            ui,
                            env!("CARGO_PKG_REPOSITORY"),
                            env!("CARGO_PKG_REPOSITORY"),
                            true,
                        );
                    });
                    ui.vertical_centered(|ui| {
                        ui.label("Homepage: ");
                        make_hyperlink(
                            ui,
                            env!("CARGO_PKG_HOMEPAGE"),
                            env!("CARGO_PKG_HOMEPAGE"),
                            true,
                        );
                    });
                    ui.separator();
                    ui.label("Authors:");
                    for one_author in env!("CARGO_PKG_AUTHORS").split(":") {
                        ui.label(format!("{}", one_author));
                    }
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(format!(
                        "egui is an immediate mode GUI library written in Rust. egui runs both on the web and natively on {}{}{}. \
                        On the web it is compiled to WebAssembly and rendered with WebGL.{}",
                        OS_APPLE, OS_LINUX, OS_WINDOWS,
                        if cfg!(target_arch = "wasm32") {
                            " Everything you see is rendered as textured triangles. There is no DOM, HTML, JS or CSS. Just Rust."
                        } else {""}
                    ));
                });
            });
    }
}

impl Default for TramexApp {
    fn default() -> Self {
        Self {
            frontend: FrontEnd::new(),
            error_panel: None,
            show_about: false,
        }
    }
}

impl eframe::App for TramexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.menu_bar(ctx, ui);
                self.frontend.menu_bar(ui);
            });
        });

        if let Err(err) = self.frontend.ui_connector(ctx) {
            self.error_panel = Some(err);
        }

        if let Err(err) = self.frontend.ui(ctx) {
            self.error_panel = Some(err);
        }

        self.ui_error_panel(ctx);
        if self.show_about {
            self.ui_about_windows(ctx);
        }
    }
}
