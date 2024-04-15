use eframe::egui;
use poll_promise::Promise;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FileHandler {
    #[serde(skip)]
    pub picked_path: Option<String>,
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<(String, String), String>>>,
    pub is_open: bool,
    error: Option<String>,
}

impl FileHandler {
    pub fn new() -> Self {
        Self {
            picked_path: None,
            file_upload: None,
            is_open: false,
            error: None,
        }
    }
    pub fn get_result(&mut self) -> Result<String, String> {
        return Err("Not implemented".to_string());
    }
    fn handle_dialog(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.error = None;
            self.file_upload = Some(Promise::spawn_local(async {
                let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
                if let Some(file) = file_selected {
                    let buf = file.read().await;
                    return match std::str::from_utf8(&buf) {
                        Ok(v) => Ok((v.to_string(), file.file_name())),
                        Err(e) => Err(e.to_string()),
                    };
                }
                Err("No file Selected".to_string())
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.file_upload = Some(Promise::spawn_thread("slow", move || {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    // read file as string
                    if let Some(path) = path.to_str() {
                        let path = path.to_string();
                        let buf = std::fs::read(path.clone());
                        let buf = match buf {
                            Ok(v) => v,
                            Err(e) => {
                                log::warn!("{:?}", e);
                                return Err(e.to_string());
                            }
                        };
                        return match std::str::from_utf8(&buf) {
                            Ok(v) => Ok((v.to_string(), path)),
                            Err(e) => Err(e.to_string()),
                        };
                    }
                }
                Err("No file Selected".to_string())
            }))
        }
    }
}

impl super::PanelController for FileHandler {
    fn name(&self) -> &'static str {
        "File Handler"
    }
    fn window_title(&self) -> &'static str {
        "File Handler"
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

impl super::PanelView for FileHandler {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open file…").clicked() {
            self.handle_dialog();
        }
        if self.picked_path.is_none() {
            if let Some(result) = &self.file_upload {
                if let Some(ready) = result.ready() {
                    if let Ok(file) = &ready {
                        self.picked_path = Some(file.1.clone());
                    } else if let Err(e) = ready {
                        self.error = Some(e.to_owned());
                    }
                }
            }
        }

        if let Some(picked_path) = &self.picked_path {
            ui.horizontal(|ui| {
                ui.label("Picked file:");
                ui.monospace(picked_path);
            });
        } else if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        } else if self.file_upload.is_some() {
            ui.add(egui::Spinner::new());
        }
    }
}
