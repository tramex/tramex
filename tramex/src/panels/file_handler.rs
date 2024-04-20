use eframe::egui;
use poll_promise::Promise;
use tramex_tools::{errors::TramexError, file_handler::File};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FileHandler {
    #[serde(skip)]
    pub picked_path: Option<String>,
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<File, TramexError>>>,
}

impl FileHandler {
    pub fn new() -> Self {
        Self {
            picked_path: None,
            file_upload: None,
        }
    }

    pub fn reset(&mut self) {
        self.picked_path = None;
        self.file_upload = None;
    }

    pub fn clear(&mut self) {
        self.file_upload = None;
    }

    pub fn get_result(&mut self) -> Result<File, TramexError> {
        return match &self.file_upload {
            Some(result) => match &result.ready() {
                Some(ready) => match ready {
                    Ok(curr_file) => Ok(curr_file.clone()),
                    Err(e) => Err(e.to_owned()),
                },
                None => Err(TramexError::new(
                    "File not ready".to_string(),
                    tramex_tools::errors::ErrorCode::FileNotReady,
                )),
            },
            None => Err(TramexError::new(
                "No file selected".to_string(),
                tramex_tools::errors::ErrorCode::FileNoFileSelected,
            )),
        };
    }

    pub fn get_picket_path(&self) -> Option<String> {
        self.picked_path.clone()
    }

    fn handle_dialog(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.file_upload = Some(Promise::spawn_local(async {
                let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
                if let Some(curr_file) = file_selected {
                    let buf = curr_file.read().await;
                    log::info!("File readed from wasm");
                    return match std::str::from_utf8(&buf) {
                        Ok(v) => Ok(File::new(curr_file.file_name().into(), v.to_string())),
                        Err(e) => Err(TramexError::new(
                            e.to_string(),
                            tramex_tools::errors::ErrorCode::FileInvalidEncoding,
                        )),
                    };
                }
                Err(TramexError::new(
                    "No file Selected".to_string(),
                    tramex_tools::errors::ErrorCode::FileNoFileSelected,
                ))
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.file_upload = Some(Promise::spawn_thread("slow", move || {
                if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                    // read file as string
                    if let Some(path) = path_buf.to_str() {
                        let path = path.to_string();
                        let buf = std::fs::read(path.clone());
                        let buf = match buf {
                            Ok(v) => v,
                            Err(e) => {
                                log::warn!("{:?}", e);
                                return Err(TramexError::new(
                                    e.to_string(),
                                    tramex_tools::errors::ErrorCode::FileErrorReadingFile,
                                ));
                            }
                        };
                        return match std::str::from_utf8(&buf) {
                            Ok(v) => Ok(File::new(path_buf, v.to_string())),
                            Err(e) => Err(TramexError::new(
                                e.to_string(),
                                tramex_tools::errors::ErrorCode::FileInvalidEncoding,
                            )),
                        };
                    }
                }
                Err(TramexError::new(
                    "No file Selected".to_string(),
                    tramex_tools::errors::ErrorCode::FileNoFileSelected,
                ))
            }))
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Result<bool, TramexError> {
        if ui.button("Open file…").clicked() {
            self.handle_dialog();
        }

        match self.check_file_load() {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        if let Some(picked_path) = &self.picked_path {
            ui.horizontal(|ui| {
                ui.label("Picked file:");
                ui.monospace(picked_path);
            });
            return Ok(true);
        } else if self.file_upload.is_some() {
            ui.add(egui::Spinner::new());
        }
        return Ok(false);
    }

    pub fn check_file_load(&mut self) -> Result<(), TramexError> {
        if self.picked_path.is_none() {
            if let Some(result) = &self.file_upload {
                if let Some(ready) = result.ready() {
                    match &ready {
                        Ok(file) => {
                            let path_filename = file
                                .file_path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(|f| f.to_string())
                                .unwrap_or_default();
                            self.picked_path = Some(path_filename);
                        }
                        Err(e) => {
                            return Err(e.to_owned());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
