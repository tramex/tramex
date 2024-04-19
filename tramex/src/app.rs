use eframe::egui::{self};
use tramex_tools::errors::TramexError;
use tramex_tools::types::internals::Interface;

use crate::frontend::FrontEnd;
use crate::make_hyperlink;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use crate::panels::FileHandler;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ExampleApp {
    pub url: String,
    #[serde(skip)]
    frontend: Option<FrontEnd>,
    #[serde(skip)]
    file_upload: Option<FileHandler>,
    #[serde(skip)]
    error_panel: Option<TramexError>,
}

impl ExampleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    fn menu_bar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);
        ui.separator();
        ui.menu_button("File", |ui| {
            if ui.button("Upload a file").clicked() {
                // TODO open file dialog
                if self.file_upload.is_none() {
                    self.file_upload = Some(FileHandler::new());
                } else {
                    self.file_upload = None;
                }
            }
            if ui.button("Organize windows").clicked() {
                ui.ctx().memory_mut(|mem| mem.reset_areas());
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

    fn ui_file_handler(&mut self, ctx: &egui::Context) {
        if let Some(file_handle) = &mut self.file_upload {
            use crate::panels::PanelController; // to use show();
            let mut file_handle_open = true;
            file_handle.show(ctx, &mut file_handle_open);
            if let Ok(result) = file_handle.get_result() {
                log::info!("File upload result: {:?}", result);
                // create fake websocket handler
                // self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.file_upload = None;
            }
            if !file_handle_open {
                log::debug!("Closing file windows");
                self.file_upload = None;
            }
        }
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
                    ui.colored_label(egui::Color32::RED, &error_item.message);
                    if ui.button("Copy error").clicked() {
                        ui.output_mut(|o| o.copied_text = error_item.message.clone());
                    };
                    make_hyperlink(
                        ui,
                        "Report the issue",
                        "https://github.com/tramex/tramex/issues/new",
                        true,
                    );
                    if error_item.recoverable {
                        ui.label("Recoverable error !");
                    }
                });
            if error_item.recoverable && !error_panel_open {
                log::debug!("Closing file windows");
                self.error_panel = None;
            }
        }
    }
}

impl Default for ExampleApp {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:9001".to_owned(),
            frontend: None,
            file_upload: None,
            error_panel: None,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.menu_bar(ctx, ui);
                if let Some(current_frontend) = &mut self.frontend {
                    current_frontend.menu_bar(ui);
                }
            });
        });

        egui::TopBottomPanel::top("server").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                if let Some(curr_front) = &self.frontend {
                    ui.label(&self.url);
                    if ui.button("Close").clicked() {
                        // close connection
                        if let Interface::Ws(interface_ws) =
                            &mut curr_front.connector.borrow_mut().interface
                        {
                            if let Err(err) = interface_ws.ws_sender.close() {
                                log::error!("Error closing WebSocket: {}", err);
                            }
                        }
                        self.frontend = None;
                    }
                } else {
                    if (ui.text_edit_singleline(&mut self.url).lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Connect").clicked()
                    {
                        let mut frontend = FrontEnd::new();
                        let new_ctx = ctx.clone();
                        let wakeup = move || new_ctx.request_repaint(); // wake up UI thread on new message
                        if let Err(err) = frontend.connect(&self.url, wakeup) {
                            self.error_panel = Some(err);
                        }
                        self.frontend = Some(frontend);
                    }
                }
            });
        });

        if let Some(frontend) = &mut self.frontend {
            if let Err(err) = frontend.ui(ctx) {
                self.error_panel = Some(err);
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.horizontal(|ui| ui.vertical(|_ui| {})));
        }
        self.ui_file_handler(ctx);
        self.ui_error_panel(ctx);
    }
}
