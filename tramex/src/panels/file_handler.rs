use std::path::Path;

use eframe::egui;
use poll_promise::Promise;
use tramex_tools::{errors::TramexError, file_handler::File};

#[derive(Debug, serde::Deserialize)]
pub struct Item {
    name: String,
    list: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FileHandler {
    #[serde(skip)]
    pub picked_path: Option<String>,
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<File, TramexError>>>,
    #[serde(skip)]
    pub file_list: Option<Promise<Result<Vec<Item>, TramexError>>>,
}

impl FileHandler {
    pub fn new() -> Self {
        let file_list = Some(Promise::spawn_local(async {
            let request = ehttp::Request::get(
                "https://raw.githubusercontent.com/tramex/files/main/list.json?raw=true",
            );
            let res = ehttp::fetch_async(request).await;
            match res {
                Ok(res) => {
                    log::info!("File list fetched");
                    let items: Result<Vec<Item>, serde_json::Error> =
                        serde_json::from_slice(&res.bytes);
                    match items {
                        Ok(items) => Ok(items),
                        Err(e) => {
                            log::warn!("{:?}", e);
                            return Err(TramexError::new(
                                e.to_string(),
                                tramex_tools::errors::ErrorCode::FileErrorReadingFile,
                            ));
                        }
                    }
                }
                Err(e) => {
                    log::warn!("{:?}", e);
                    return Err(TramexError::new(
                        e.to_string(),
                        tramex_tools::errors::ErrorCode::FileErrorReadingFile,
                    ));
                }
            }
        }));
        Self {
            picked_path: None,
            file_upload: None,
            file_list,
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

        if let Some(result) = &self.file_list {
            if let Some(ready) = result.ready() {
                match &ready {
                    Ok(items) => {
                        ui.vertical(|ui| {
                            ui.label("Files list:");
                            ui.add_space(1.0);
                            for item in items {
                                ui.collapsing(&item.name, |ui| {
                                    for sub_item in &item.list {
                                        let path = match Path::new(sub_item).file_name() {
                                            Some(f) => f.to_str().unwrap_or(sub_item),
                                            None => sub_item,
                                        };
                                        if ui.monospace(format!("{}", path)).clicked() {
                                            log::info!("File selected: {}", sub_item);
                                        }
                                    }
                                });
                            }
                        });
                    }
                    Err(e) => {
                        return Err(e.to_owned());
                    }
                }
            }
        }

        ui.add_space(12.0);
        if let Some(picked_path) = &self.picked_path {
            ui.horizontal(|ui| {
                ui.label("Picked file:");
                ui.monospace(picked_path);
            });
            return Ok(true);
        } else if self.file_upload.is_some() {
            ui.add(egui::Spinner::new());
        }
        ui.add_space(12.0);
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
