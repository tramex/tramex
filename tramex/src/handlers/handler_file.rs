//! File handler panel
use std::path::Path;

use eframe::egui;
use poll_promise::Promise;
use tramex_tools::{
    data::Data,
    errors::TramexError,
    interface::{interface_file::file_handler::File, interface_types::InterfaceTrait},
    tramex_error,
};

use super::Handler;

#[derive(Debug, serde::Deserialize)]
/// Item to show in the file list
struct Item {
    /// Name of the item
    name: String,

    /// List of files
    list: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
/// File handler
pub struct FileHandler {
    #[serde(skip)]
    /// Picked path
    pub picked_path: Option<String>,
    #[serde(skip)]
    /// File upload
    pub file_upload: Option<Promise<Result<File, TramexError>>>,
    #[serde(skip)]
    /// File list
    file_list: Option<Promise<Result<Vec<Item>, TramexError>>>,

    /// URL files
    url_files: String,

    #[serde(skip)]
    /// File
    file: Option<File>,
}

impl FileHandler {
    /// Create a new file handler
    pub fn new() -> Self {
        let url_list = if let Some(url_f) = FileHandler::parse_current_url() {
            url_f
        } else {
            "https://raw.githubusercontent.com/tramex/files/main/list.json?raw=true".into()
        };

        let mut s = Self {
            picked_path: None,
            file_upload: None,
            file_list: None,
            url_files: url_list,
            file: None,
        };
        s.get_file_list();
        s
    }

    /// Parse the current URL to get the files list URL
    fn parse_current_url() -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            match web_sys::window() {
                Some(window) => {
                    let location = window.location();
                    if let Ok(href) = location.href() {
                        if let Ok(url) = web_sys::Url::new(&href) {
                            let search_params = url.search_params();
                            if let Some(url_f) = search_params.get("files_url") {
                                return Some(url_f);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Get the files list
    pub fn get_file_list(&mut self) {
        self.file_list = None;
        let callback = move |res: Result<ehttp::Response, String>| match res {
            Ok(res) => {
                log::info!("File list fetched");
                let items: Result<Vec<Item>, serde_json::Error> = serde_json::from_slice(&res.bytes);
                match items {
                    Ok(items) => Ok(items),
                    Err(e) => {
                        log::warn!("{:?}", e);
                        Err(tramex_error!(
                            format!("Error decoding files list: {}", e.to_string()),
                            tramex_tools::errors::ErrorCode::FileErrorReadingFile
                        ))
                    }
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                Err(tramex_error!(
                    format!("Error loading files list: {}", e.to_string()),
                    tramex_tools::errors::ErrorCode::RequestError
                ))
            }
        };
        let request = ehttp::Request::get(&self.url_files);
        #[cfg(target_arch = "wasm32")]
        {
            self.file_list = Some(Promise::spawn_local(async move {
                let res = ehttp::fetch_async(request).await;
                callback(res)
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.file_list = Some(Promise::spawn_thread("http_get", move || {
                let res = ehttp::fetch_blocking(&request);
                callback(res)
            }));
        }
    }

    /// Reset the file handler
    pub fn reset(&mut self) {
        self.picked_path = None;
        self.file_upload = None;
    }

    /// Clear the file handler
    pub fn clear(&mut self) {
        self.file_upload = None;
        self.file_list = None;
    }

    /// Get the result
    /// # Errors
    /// Return an error if the file contains errors
    pub fn get_result(&mut self) -> Result<Option<File>, TramexError> {
        let mut should_clean = false;
        let res = match &self.file_upload {
            Some(result) => match &result.ready() {
                Some(ready) => match ready {
                    Ok(curr_file) => Ok(Some(curr_file.clone())),
                    Err(e) => {
                        should_clean = true;
                        Err(e.to_owned())
                    }
                },
                None => Err(tramex_error!(
                    "File not ready".to_string(),
                    tramex_tools::errors::ErrorCode::FileNotReady
                )),
            },
            None => Ok(None),
        };
        log::debug!("Result: {:?}", res);
        if should_clean {
            log::debug!("Cleaning file upload");
            self.clear();
        }
        res
    }

    /// Load file from URL
    pub fn load_from_url(&mut self, url: String) {
        self.reset();
        let copied_url = url.clone();
        let call = move |res: Result<ehttp::Response, String>| match res {
            Ok(res) => {
                log::info!("File fetched");
                match std::str::from_utf8(&res.bytes) {
                    Ok(v) => {
                        let path = match Path::new(&url).file_name() {
                            Some(f) => f.to_str().unwrap_or(&url),
                            None => &url,
                        };
                        Ok(File::new(path.into(), v.to_string()))
                    }
                    Err(e) => Err(tramex_error!(
                        e.to_string(),
                        tramex_tools::errors::ErrorCode::FileInvalidEncoding
                    )),
                }
            }
            Err(e) => {
                log::warn!("{:?}", e);
                Err(tramex_error!(
                    e.to_string(),
                    tramex_tools::errors::ErrorCode::FileErrorReadingFile
                ))
            }
        };

        let request = ehttp::Request::get(copied_url);
        #[cfg(target_arch = "wasm32")]
        {
            self.file_upload = Some(Promise::spawn_local(async move {
                let res = ehttp::fetch_async(request).await;
                call(res)
            }));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.file_upload = Some(Promise::spawn_thread("http_get", move || {
                let res = ehttp::fetch_blocking(&request);
                call(res)
            }));
        }
    }

    /// Get the picked path
    pub fn get_picket_path(&self) -> Option<String> {
        self.picked_path.clone()
    }

    /// Load file upload
    fn load_file_upload(&mut self) {
        self.reset();
        #[cfg(target_arch = "wasm32")]
        {
            self.file_upload = Some(Promise::spawn_local(async {
                let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
                if let Some(curr_file) = file_selected {
                    let buf = curr_file.read().await;
                    log::info!("File reading from wasm");
                    return match std::str::from_utf8(&buf) {
                        Ok(v) => Ok(File::new(curr_file.file_name().into(), v.to_string())),
                        Err(e) => Err(tramex_error!(
                            e.to_string(),
                            tramex_tools::errors::ErrorCode::FileInvalidEncoding
                        )),
                    };
                }
                Err(tramex_error!(
                    "Upload: no file Selected".to_string(),
                    tramex_tools::errors::ErrorCode::FileNotSelected
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
                                return Err(tramex_error!(
                                    e.to_string(),
                                    tramex_tools::errors::ErrorCode::FileErrorReadingFile
                                ));
                            }
                        };
                        return match std::str::from_utf8(&buf) {
                            Ok(v) => Ok(File::new(path_buf, v.to_string())),
                            Err(e) => Err(tramex_error!(
                                e.to_string(),
                                tramex_tools::errors::ErrorCode::FileInvalidEncoding
                            )),
                        };
                    }
                }
                Err(tramex_error!(
                    "Upload: no file Selected".to_string(),
                    tramex_tools::errors::ErrorCode::FileNotSelected
                ))
            }))
        }
    }

    /// Render the file upload
    /// # Errors
    /// Return an error if the file contains errors
    pub fn internal_ui(&mut self, ui: &mut egui::Ui) -> Result<(), TramexError> {
        let mut error_to_return = None;
        ui.add_enabled_ui(self.get_picket_path().is_none(), |ui| {
            if ui.button("Open file…").clicked() {
                self.load_file_upload();
            }

            match self.check_file_load() {
                Ok(_) => {}
                Err(e) => {
                    error_to_return = Some(e);
                }
            }
            let mut file_path = None;
            if let Some(result) = &self.file_list {
                if let Some(ready) = result.ready() {
                    match &ready {
                        Ok(items) => {
                            ui.vertical(|ui| {
                                ui.collapsing("Files list:", |ui| {
                                    ui.add_space(1.0);
                                    for item in items {
                                        ui.collapsing(&item.name, |ui| {
                                            for sub_item in &item.list {
                                                let path = match Path::new(sub_item).file_name() {
                                                    Some(f) => f.to_str().unwrap_or(sub_item),
                                                    None => sub_item,
                                                };
                                                if ui.button(path).clicked() {
                                                    log::info!("File selected: {}", sub_item);
                                                    file_path = Some(sub_item.to_string());
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        }
                        Err(e) => {
                            error_to_return = Some(e.to_owned());
                            self.clear();
                        }
                    }
                } else {
                    ui.label("Loading files list...");
                    ui.spinner();
                }
            } else {
                ui.label("No files list");
            }
            if let Some(filepath) = &file_path {
                self.load_from_url(filepath.to_string());
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            } else if self.file_upload.is_some() {
                ui.add(egui::Spinner::new());
            }
            ui.add_space(12.0);
        });
        if let Some(err) = error_to_return {
            return Err(err);
        };
        Ok(())
    }

    /// Check file load
    /// # Errors
    /// Return an error if the file contains errors
    pub fn check_file_load(&mut self) -> Result<(), TramexError> {
        if self.picked_path.is_none() {
            let mut error = None;
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
                            error = Some(e.to_owned());
                        }
                    }
                }
            }
            if let Some(e) = error {
                self.clear();
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Handler for FileHandler {
    fn ui_options(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("File Options", |ui| {
            ui.label("Index of files URL:");
            ui.add(egui::TextEdit::singleline(&mut self.url_files));
            if ui.button("Reload list").clicked() {
                self.get_file_list();
            }
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, data: &mut Data, _new_ctx: egui::Context) -> Result<bool, TramexError> {
        self.internal_ui(ui)?; // may return error
        if self.picked_path.is_some() && self.file.is_none() {
            match self.get_result() {
                Ok(Some(curr_file)) => {
                    self.file = curr_file.into();
                    self.clear();
                }
                Ok(None) => {}
                Err(err) => {
                    log::error!("Error in get_result() {:?}", err);
                    self.clear();
                    return Err(err);
                }
            };
        };
        if self.get_picket_path().is_some() && ui.button("Close").on_hover_text("Close file").clicked() {
            self.reset();
            data.clear();
            return Ok(true);
        }
        Ok(false)
    }

    fn close(&mut self) -> Result<(), TramexError> {
        if let Some(file) = &mut self.file {
            file.close()?;
        }
        Ok(())
    }

    fn get_more_data(
        &mut self,
        layer_list: tramex_tools::interface::layer::Layers,
        data: &mut tramex_tools::data::Data,
    ) -> Result<(), Vec<TramexError>> {
        if let Some(file) = &mut self.file {
            return file.get_more_data(layer_list, data);
        }
        Ok(())
    }

    fn try_recv(&mut self, _data: &mut tramex_tools::data::Data) -> Result<(), Vec<TramexError>> {
        Ok(())
    }

    fn show_available(&self, ui: &mut egui::Ui) {
        if self.file.is_some() {
            ui.label("File available");
        } else {
            ui.label("File not available");
        }
    }

    fn is_full_read(&self) -> bool {
        if let Some(file) = &self.file {
            return file.full_read;
        }
        false
    }

    fn is_interface(&self) -> bool {
        self.file.is_some()
    }

    fn is_interface_available(&self) -> bool {
        if let Some(file) = &self.file {
            return file.available;
        }
        false
    }
}
