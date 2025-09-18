//! Frontend module
use crate::handlers::Handler;
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;

use crate::panels::{
    PanelController, logical_channels::LogicalChannels, panel_message::MessageBox, rrc_status::LinkPanel,
    trame_manager::TrameManager,
};
use crate::set_open;
use egui::Ui;
use std::collections::BTreeSet;
use tramex_tools::data::Data;
use tramex_tools::errors::{ErrorCode, TramexError};

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
    /// Data
    #[serde(skip)]
    pub data: Data,

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
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            data: Data::default(),
            open_windows: BTreeSet::new(),
            windows: Vec::new(),
            open_menu_connector: true,
            radio_choice: Choice::default(),
            handler: None,
            trame_manager: TrameManager::new(),
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
            open_windows,
            windows: wins,
            ..Default::default()
        }
    }

    /// Menu bar
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.interface_available() {
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    if let Some(handle) = &self.handler {
                        self.trame_manager.show_controls(ui, &mut self.data, handle.is_full_read());
                    }
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
    /// # Errors
    /// Return a vector of TramexError
    pub fn ui_connector(&mut self, ctx: &egui::Context) -> Result<(), Vec<TramexError>> {
        let mut errors = vec![];
        if self.open_menu_connector {
            egui::SidePanel::left("backend_panel")
                .resizable(false)
                .show_animated(ctx, self.open_menu_connector, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Connector");
                        let save = self.radio_choice.clone();
                        ui.horizontal(|ui| {
                            let enabled = if let Some(handle) = &self.handler {
                                !handle.is_interface()
                            } else {
                                true
                            };
                            ui.add_enabled_ui(enabled, |ui| {
                                ui.label("Choose ws or file");
                                ui.radio_value(&mut self.radio_choice, Choice::File, "File");
                                #[cfg(feature = "websocket")]
                                ui.radio_value(&mut self.radio_choice, Choice::WebSocket, "WebSocket");
                            });
                        });
                        match &self.radio_choice {
                            #[cfg(feature = "websocket")]
                            Choice::WebSocket => {
                                if save != self.radio_choice || self.handler.is_none() {
                                    self.handler = Some(Box::new(WsHandler::new()));
                                }
                            }
                            Choice::File => {
                                if save != self.radio_choice || self.handler.is_none() {
                                    self.handler = Some(Box::new(FileHandler::new()));
                                }
                            }
                        };
                        ui.vertical(|ui| {
                            if let Some(handle) = &mut self.handler {
                                match handle.ui(ui, &mut self.data, ctx.clone()) {
                                    Ok(true) => {
                                        self.handler = None;
                                        for one_panel in self.windows.iter_mut() {
                                            one_panel.clear();
                                        }
                                    }
                                    Ok(false) => {}
                                    Err(err) => {
                                        errors.push(err);
                                    }
                                }
                            }
                        });
                    });
                    ui.separator();
                    if let Some(handle) = &mut self.handler {
                        handle.ui_options(ui);
                    }
                    if self.interface_available() {
                        if let Some(handle) = &mut self.handler {
                            self.trame_manager.show_options(ui);
                            if self.trame_manager.should_get_more_log {
                                self.trame_manager.should_get_more_log = false;
                                if let Err(err) =
                                    handle.get_more_data(self.trame_manager.layers_list.clone(), &mut self.data)
                                {
                                    for one_error in err {
                                        if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                                            errors.push(one_error);
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }

    /// Check if the interface is available
    pub fn interface_available(&self) -> bool {
        if let Some(handle) = &self.handler {
            handle.is_interface_available()
        } else {
            false
        }
    }

    /// Show the UI
    /// # Errors
    /// Return a vector of TramexError
    pub fn ui(&mut self, ctx: &egui::Context) -> Result<(), Vec<TramexError>> {
        let mut error_to_return = vec![];
        if let Some(handle) = &mut self.handler {
            if let Err(errors_vect) = handle.try_recv(&mut self.data) {
                for one_error in errors_vect {
                    if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                        error_to_return.push(one_error);
                    }
                }
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.interface_available() {
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    if let Err(err) = one_window.show(ctx, &mut is_open, &mut self.data) {
                        log::error!("Error in window {}", one_window.name());
                        error_to_return.push(err);
                    }
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
                // show nothing
            } else {
                match &self.handler {
                    Some(handle) => handle.show_available(ui),
                    None => {
                        ui.label("Not connected");
                    }
                };
            }
        });
        if !error_to_return.is_empty() {
            return Err(error_to_return);
        }
        Ok(())
    }
}
