//! Frontend module
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;
use crate::handlers::Handler;

use crate::panels::{
    logical_channels::LogicalChannels, panel_message::MessageBox, rrc_status::LinkPanel, trame_manager::TrameManager,
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
    handler: Option<Box<dyn Handler>>,

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
            handler: None,
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
        let status = LinkPanel::new();
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<LinkPanel>::new(status),
        ];
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
    pub fn ui_options(&mut self, ui: &mut egui::Ui) {
        match &mut self.handler {
            Some(handler) => {
                handler.ui_options(ui);
                ui.label("Index of files URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_files));
            }
            None => {}
        }
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
                                if save != self.radio_choice || self.handler.is_none() {
                                    self.handler = Some(Box::new(WsHandler::new()));
                                }
                                if let Some(handler) = &mut self.handler {
                                    if let Err(err) = handler.ui(ui, &mut self.connector, ctx.clone()) {
                                        error = Some(err);
                                    }
                                }
                            }
                            Choice::File => {
                                if save != self.radio_choice || self.handler.is_none() {
                                    self.handler = Some(Box::new(FileHandler::new(&self.url_files)));
                                }
                                if let Some(file_handle) = &mut self.handler {
                                    if let Err(err) = file_handle.ui(ui, &mut self.connector, ctx.clone()) {
                                        error = Some(err);
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
        let mut error_to_return: Option<TramexError> = None;
        if self.connector.interface.is_some() {
            if let Err(err) = self.connector.try_recv() {
                error_to_return = Some(err);
            }
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
