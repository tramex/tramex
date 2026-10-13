//! The main application state and logic.

use eframe::egui::{self};
use tramex_tools::errors::TramexError;

use crate::frontend::FrontEnd;
use crate::make_hyperlink;

#[cfg(feature = "ai")]
use crate::ai_settings::AISettings;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
/// The main application state
pub struct TramexApp {
    /// Frontend
    pub frontend: FrontEnd,

    #[serde(skip)]
    /// Error panel
    error_panel: Vec<TramexError>,

    /// Show about windows
    show_about_windows: bool,

    #[serde(skip)]
    /// Show settings window
    show_settings_window: bool,

    #[serde(skip)]
    /// Active tab in the settings window
    settings_tab: SettingsTab,

    #[cfg(feature = "ai")]
    /// AI settings (provider, API key)
    pub ai_settings: AISettings,
}

/// Active tab in the Settings window
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    /// Layer filters
    #[default]
    Layers,
    /// File loading options
    File,
    /// AI configuration
    #[cfg(feature = "ai")]
    Ai,
}

impl TramexApp {
    /// Load the app state from the given storage.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply the tramex theme (colors, spacing, text styles) for both modes
        crate::theme::ThemeColors::apply_style(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self { ..Default::default() }
    }

    /// Save the app state to the given storage.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_theme_preference_switch(ui);
        ui.separator();
        ui.menu_button("Menu", |ui| {
            if ui.button("Organize windows").clicked() {
                ui.ctx().memory_mut(|mem| mem.reset_areas());
            }
            if ui.button("About").clicked() {
                self.show_about_windows = !self.show_about_windows;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });

        // Windows menu button
        self.frontend.menu_bar(ui);

        ui.menu_button("Settings", |ui| {
            #[cfg(feature = "ai")]
            if ui.button("AI Configuration").clicked() {
                self.settings_tab = SettingsTab::Ai;
                self.show_settings_window = true;
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("Layers").clicked() {
                self.settings_tab = SettingsTab::Layers;
                self.show_settings_window = true;
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("File options").clicked() {
                self.settings_tab = SettingsTab::File;
                self.show_settings_window = true;
                ui.close_kind(egui::UiKind::Menu);
            }
        });

        ui.menu_button("About", |ui| {
            make_hyperlink(ui, "User documentation", "https://tramex.github.io/tramex/docs/", true);
            make_hyperlink(ui, "tramex types", "https://tramex.github.io/tramex/crates/tramex/", true);
            make_hyperlink(
                ui,
                "tramex-tools types",
                "https://tramex.github.io/tramex/crates/tramex_tools/",
                true,
            );
            make_hyperlink(ui, "tramex repository", "https://github.com/tramex/tramex", true);
        });
    }

    /// Display the error panel
    fn ui_error_panel(&mut self, ctx: &egui::Context) {
        if !self.error_panel.is_empty() {
            let mut error_panel_open: bool = true;
            egui::Window::new("Errors")
                .default_width(320.0)
                .default_height(480.0)
                .open(&mut error_panel_open)
                .resizable([true, false])
                .show(ctx, |ui| {
                    let mut to_clean = vec![];
                    egui::ScrollArea::vertical()
                        .id_salt("scroll_area_errors")
                        .max_height(400.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            egui::Grid::new("error_ui").min_col_width(60.0).show(ui, |ui| {
                                for (idx, one_error) in self.error_panel.iter().enumerate() {
                                    if show_error(ui, one_error) {
                                        to_clean.push(idx);
                                    }
                                    ui.end_row();
                                }
                            });
                        });
                    for idx in to_clean.iter().rev() {
                        self.error_panel.remove(*idx);
                    }
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        make_hyperlink(ui, "Report an issue", "https://github.com/tramex/tramex/issues/new", true);
                    });
                });
            if !error_panel_open {
                log::debug!("Closing error windows");
                self.error_panel.clear();
            }
        }
    }

    /// Display the settings window
    fn ui_settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings_window;
        egui::Window::new("Settings")
            .open(&mut open)
            .resizable([true, true])
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    #[cfg(feature = "ai")]
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Ai, "AI");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Layers, "Layers");
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::File, "File options");
                });
                ui.separator();
                match self.settings_tab {
                    #[cfg(feature = "ai")]
                    SettingsTab::Ai => self.ai_settings.ui(ui),
                    SettingsTab::Layers => self.frontend.show_layer_options(ui),
                    SettingsTab::File => self.frontend.show_file_options(ui),
                }
            });
        self.show_settings_window = open;
    }

    /// Display the about windows
    fn ui_about_windows(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about_windows)
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
                        make_hyperlink(ui, env!("CARGO_PKG_REPOSITORY"), env!("CARGO_PKG_REPOSITORY"), true);
                    });
                    ui.vertical_centered(|ui| {
                        ui.label("Homepage: ");
                        make_hyperlink(ui, env!("CARGO_PKG_HOMEPAGE"), env!("CARGO_PKG_HOMEPAGE"), true);
                    });
                    ui.separator();
                    ui.label("Authors:");
                    for one_author in env!("CARGO_PKG_AUTHORS").split(':') {
                        ui.label(one_author.to_string());
                    }
                    ui.separator();
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Learn more about tramex here: ");
                        make_hyperlink(
                            ui,
                            "https://tramex.github.io/tramex/docs/",
                            "https://tramex.github.io/tramex/docs/",
                            true,
                        );
                    });
                });
            });
    }
}

impl Default for TramexApp {
    fn default() -> Self {
        Self {
            frontend: FrontEnd::new(),
            error_panel: vec![],
            show_about_windows: false,
            show_settings_window: false,
            settings_tab: SettingsTab::default(),
            #[cfg(feature = "ai")]
            ai_settings: AISettings::default(),
        }
    }
}

impl eframe::App for TramexApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.menu_bar(ui);
            });
        });

        if let Err(err) = &mut self.frontend.ui_connector(ui) {
            self.error_panel.append(err);
        }

        if let Err(err) = &mut self.frontend.ui(ui) {
            self.error_panel.append(err);
        }

        self.ui_error_panel(ui.ctx());
        if self.show_about_windows {
            self.ui_about_windows(ui.ctx());
        }

        if self.show_settings_window {
            self.ui_settings_window(ui.ctx());
        }

        #[cfg(feature = "ai")]
        {
            self.ai_settings.try_load_env_key();
            self.frontend
                .set_ai_config(&self.ai_settings.api_key, &self.ai_settings.provider, &self.ai_settings.model);
        }
    }
}

/// Display an error
fn show_error(ui: &mut egui::Ui, error_item: &TramexError) -> bool {
    let colors = crate::theme::ThemeColors::get(ui);
    ui.colored_label(colors.error, &error_item.message)
        .on_hover_text_at_pointer(&error_item.debug);
    if ui.button("Copy error").clicked() {
        ui.ctx()
            .copy_text(format!("{}\n{}", &error_item.get_msg(), &error_item.message));
    };
    if ui.button("Close this error").clicked() {
        return true;
    }
    false
}
