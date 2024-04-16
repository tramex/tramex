use eframe::egui::{self};

use crate::frontend::FrontEnd;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use crate::panels::FileHandler;
use crate::{make_hyperlink, set_open};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ExampleApp {
    pub url: String,
    #[serde(skip)]
    pub error: String,
    #[serde(skip)]
    frontend: Option<FrontEnd>,
    file_upload: Option<FileHandler>,
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
    fn menu_bart(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
        ui.menu_button("About", |ui| {
            make_hyperlink(
                ui,
                "General documentation",
                "https://tramex.github.io/tramex/docs/",
                true,
            );
            make_hyperlink(
                ui,
                "Rust types documentation",
                "https://docs.rs/crate/tramex/latest",
                true,
            );
            make_hyperlink(ui, "Repository", "https://github.com/tramex/tramex", true);
        });
    }
}

impl Default for ExampleApp {
    fn default() -> Self {
        Self {
            url: "ws://137.194.194.51:9001".to_owned(),
            error: Default::default(),
            frontend: None,
            file_upload: None,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.menu_bart(ctx, ui);
                if let Some(current_frontend) = &mut self.frontend {
                    if current_frontend.connector.borrow().available {
                        ui.menu_button("Windows", |ui| {
                            for one_window in current_frontend.windows.iter_mut() {
                                let mut is_open: bool =
                                    current_frontend.open_windows.contains(one_window.name());
                                ui.checkbox(&mut is_open, one_window.name());
                                set_open(
                                    &mut current_frontend.open_windows,
                                    one_window.name(),
                                    is_open,
                                );
                            }
                        });
                    }
                }
            });
        });

        egui::TopBottomPanel::top("server").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                if self.frontend.is_some() {
                    ui.label(&self.url);
                    if ui.button("Close").clicked() {
                        // TODO close connection
                        self.frontend = None;
                    }
                } else {
                    if (ui.text_edit_singleline(&mut self.url).lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Connect").clicked()
                    {
                        self.connect(ctx.clone());
                    }
                }
            });
        });

        if !self.error.is_empty() {
            egui::TopBottomPanel::top("error").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Error:");
                    ui.colored_label(egui::Color32::RED, &self.error);
                });
            });
        }

        if let Some(frontend) = &mut self.frontend {
            frontend.ui(ctx);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.horizontal(|ui| ui.vertical(|_ui| {})));
        }
        if let Some(fu) = &mut self.file_upload {
            use crate::panels::PanelController; // to use show();
            fu.show(ctx, &mut true);
            if let Ok(result) = fu.get_result() {
                log::info!("File upload result: {:?}", result);
                // create fake websocket handler
                // self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.file_upload = None;
            }
        }
    }
}

impl ExampleApp {
    fn connect(&mut self, ctx: egui::Context) {
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        let options = ewebsock::Options {
            max_incoming_frame_size: 500,
        };
        match ewebsock::connect_with_wakeup(&self.url, options, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.error.clear();
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", &self.url, error);
                self.error = error;
            }
        }
    }
}
