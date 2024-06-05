//! Frontend module
use crate::panels::{
    file_upload::FileHandler, logical_channels::LogicalChannels, message::MessageBox, trame_manager::TrameManager,
    PanelController,
};
use crate::set_open;
use egui::Ui;
use std::collections::BTreeSet;
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::interface_types::Interface;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Default)]
/// Choice enum
pub enum Choice {
    /// File choice
    #[default]
    File,

    /// WebSocket choice
    #[cfg(feature = "websocket")]
    WebSocket,
}

#[derive(serde::Deserialize, serde::Serialize)]
/// FrontEnd struct
pub struct FrontEnd {
    /// Connector
    pub connector: Connector,

    /// Open windows
    pub open_windows: BTreeSet<String>,

    #[serde(skip)]
    /// Windows
    pub windows: Vec<Box<dyn PanelController>>,

    /// Open menu connector
    pub open_menu_connector: bool,

    #[serde(skip)]
    /// File upload
    file_upload: Option<FileHandler>,

    /// Trame manager
    trame_manager: TrameManager,

    /// Radio choice
    pub radio_choice: Choice,

    /// URL files
    pub url_files: String,
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            connector: Connector::new(),
            open_windows: BTreeSet::new(),
            windows: Vec::new(),
            open_menu_connector: true,
            radio_choice: Choice::default(),
            file_upload: None,
            trame_manager: TrameManager::new(),
            url_files: "https://raw.githubusercontent.com/tramex/files/main/list.json?raw=true".into(),
        }
    }
}

impl FrontEnd {
    /// Create a new frontend
    pub fn new() -> Self {
        let mb = MessageBox::new();
        let lc = LogicalChannels::new();
        let wins: Vec<Box<dyn PanelController>> = vec![Box::<MessageBox>::new(mb), Box::<LogicalChannels>::new(lc)];
        let mut open_windows = BTreeSet::new();
        for one_box in wins.iter() {
            open_windows.insert(one_box.name().to_owned());
        }
        Self {
            connector: Connector::new(),
            open_windows,
            windows: wins,
            ..Default::default()
        }
    }

    /// Show tiny ui in about panel
    pub fn ui_about(&mut self, ui: &mut egui::Ui) {
        ui.label("Index of files URL:");
        ui.add(egui::TextEdit::singleline(&mut self.url_files));
    }

    /// Menu bar
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.connector.available {
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    self.trame_manager.show_controls(ui, &mut self.connector);
                    ui.menu_button("Windows", |ui| {
                        for one_window in self.windows.iter_mut() {
                            let mut is_open: bool = self.open_windows.contains(one_window.name());
                            ui.checkbox(&mut is_open, one_window.name());
                            set_open(&mut self.open_windows, one_window.name(), is_open);
                        }
                    });
                });
            });
        }
    }

    /// Show the URL
    #[cfg(feature = "websocket")]
    pub fn show_url(&mut self, ui: &mut Ui, new_ctx: egui::Context) -> Result<(), TramexError> {
        let connector = &mut self.connector;

        let current_url = connector.url.clone();

        if let Some(Interface::Ws(interface_ws)) = &mut connector.interface {
            ui.label("URL:");
            ui.label(&current_url);
            if ui.button("Close").clicked() {
                // close connection
                match interface_ws.close_impl() {
                    Ok(_) => {
                        connector.clear_interface();
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }

        match &connector.interface {
            Some(Interface::Ws(interface_ws)) => {
                if interface_ws.connecting {
                    ui.label("Connecting...");
                    ui.spinner();
                }
            }
            _ => {
                if (ui.text_edit_singleline(&mut connector.url).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Connect").clicked()
                {
                    let wakeup_fn = move || new_ctx.request_repaint(); // wake up UI thread on new message
                    let local_url = connector.url.clone();
                    connector.connect(&local_url, wakeup_fn)?
                }
            }
        }
        Ok(())
    }

    /// Show the UI connector
    pub fn ui_connector(&mut self, ctx: &egui::Context) -> Result<(), TramexError> {
        let mut error = None;
        if self.open_menu_connector {
            egui::SidePanel::left("backend_panel")
                .max_width(100.0)
                .resizable(false)
                .show_animated(ctx, self.open_menu_connector, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Connector");
                        let save = self.radio_choice.clone();
                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(self.connector.interface.is_none(), |ui| {
                                ui.label("Choose ws or file");
                                ui.radio_value(&mut self.radio_choice, Choice::File, "File");
                                #[cfg(feature = "websocket")]
                                ui.radio_value(&mut self.radio_choice, Choice::WebSocket, "WebSocket");
                            });
                        });
                        ui.vertical(|ui| match &self.radio_choice {
                            #[cfg(feature = "websocket")]
                            Choice::WebSocket => {
                                if let Err(err) = self.show_url(ui, ctx.clone()) {
                                    error = Some(err);
                                }
                            }
                            Choice::File => {
                                if save != self.radio_choice || self.file_upload.is_none() {
                                    self.file_upload = Some(FileHandler::new(&self.url_files));
                                }
                                if let Some(file_handle) = &mut self.file_upload {
                                    let is_file_path = file_handle.get_picket_path().is_some();
                                    ui.add_enabled_ui(!is_file_path, |ui| match file_handle.ui(ui) {
                                        Ok(bo) => {
                                            if bo && self.connector.interface.is_none() {
                                                match file_handle.get_result() {
                                                    Ok(curr_file) => {
                                                        self.connector.set_file(curr_file);
                                                        file_handle.clear();
                                                    }
                                                    Err(err) => {
                                                        log::error!("Error in get_result() {:?}", err);
                                                        error = Some(err);
                                                    }
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error in file_handle {:?}", err);
                                            error = Some(err);
                                            file_handle.reset();
                                            self.connector.clear_data();
                                        }
                                    });
                                    if is_file_path && ui.button("Close").on_hover_text("Close file").clicked() {
                                        file_handle.reset();
                                        self.connector.clear_data();
                                        self.connector.clear_interface();
                                    }
                                }
                            }
                        });
                    });
                    ui.separator();
                    if self.connector.available {
                        self.trame_manager.show_options(ui, &mut self.connector);
                        if self.trame_manager.should_get_more_log {
                            self.trame_manager.should_get_more_log = false;
                            if let Err(err) = self.connector.get_more_data(self.trame_manager.layers_list.clone()) {
                                error = Some(err);
                            }
                        }
                    }
                });
        }
        if let Some(e) = error {
            return Err(e);
        }
        Ok(())
    }

    /// Show the UI
    pub fn ui(&mut self, ctx: &egui::Context) -> Result<(), TramexError> {
        let mut error_to_return = None;
        if let Err(err) = self.connector.try_recv() {
            error_to_return = Some(err);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.connector.available {
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    if let Err(err) = one_window.show(ctx, &mut is_open, &mut self.connector.data) {
                        log::error!("Error in window {}", one_window.name());
                        error_to_return = Some(err);
                    }
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
                // show nothing
            } else {
                match &self.connector.interface {
                    #[cfg(feature = "websocket")]
                    Some(Interface::Ws(_interface_ws)) => {
                        ui.label("WebSocket not available");
                    }
                    Some(Interface::File(_interface_file)) => {
                        ui.label("File not available");
                    }
                    None => {
                        ui.label("Not connected");
                    }
                }
            }
        });
        match error_to_return {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
