//! Frontend module
use crate::handlers::Handler;
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;

use crate::panels::{
    PanelController, logical_channels::LogicalChannels, navigation_panel::NavigationPanel,
    panel_message::MessageBox, rrc_status::RRCStatusPanel, rrc_field_viewer::RrcFieldViewer,
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
    
    #[serde(skip)]
    /// Navigation panel (separate reference for button handling)
    nav_panel: NavigationPanel,
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
            nav_panel: NavigationPanel::new(),
        }
    }
}

impl FrontEnd {
    /// Create a new frontend
    pub fn new() -> Self {
        let mb = MessageBox::new();
        let lc = LogicalChannels::new();
        let status = RRCStatusPanel::new();
        let rrc_fields = RrcFieldViewer::new();
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<RRCStatusPanel>::new(status),
            Box::<RrcFieldViewer>::new(rrc_fields),
        ];
        let mut open_windows = BTreeSet::new();
        for one_box in wins.iter() {
            open_windows.insert(one_box.name().to_owned());
        }
        // Add Navigation panel to open windows by default
        open_windows.insert("Navigation".to_owned());
        
        Self {
            open_windows,
            windows: wins,
            ..Default::default()
        }
    }

    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.interface_available() {
            ui.menu_button("Windows", |ui| {
                // Navigation panel
                let mut nav_open = self.open_windows.contains("Navigation");
                ui.checkbox(&mut nav_open, "Navigation");
                set_open(&mut self.open_windows, "Navigation", nav_open);
                
                // Other windows
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    ui.checkbox(&mut is_open, one_window.name());
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
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
                    // Show layer options always (not just when file is loaded)
                    self.trame_manager.show_options(ui);
                    
                    if self.interface_available() {
                        if let Some(handle) = &mut self.handler {
                            // Keep loading batches until we find an enabled event or reach end of file
                            while self.trame_manager.should_get_more_log {
                                self.trame_manager.should_get_more_log = false;
                                let should_continue = self.trame_manager.continue_navigation_after_load;
                                
                                if let Err(err) =
                                    handle.get_more_data(self.trame_manager.layers_list.clone(), &mut self.data)
                                {
                                    for one_error in err {
                                        if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                                            errors.push(one_error);
                                        }
                                    }
                                    self.trame_manager.continue_navigation_after_load = false;
                                    break;  // Stop on error
                                } else if should_continue {
                                    // Data loaded successfully, continue navigation
                                    self.trame_manager.continue_navigation_after_load = false;
                                    self.trame_manager.continue_to_next_enabled(&mut self.data, handle.is_full_read());
                                    // If should_get_more_log is still true, the loop will continue
                                } else {
                                    // Just loading more data, not continuing navigation
                                    break;
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
                // Update total count in navigation panel
                if let Some(handle) = &self.handler {
                    self.nav_panel.total_count = handle.get_total_event_count();
                }
                
                // Show navigation panel separately
                let mut nav_open = self.open_windows.contains("Navigation");
                if let Err(err) = self.nav_panel.show(ctx, &mut nav_open, &mut self.data) {
                    log::error!("Error in Navigation panel");
                    error_to_return.push(err);
                }
                set_open(&mut self.open_windows, "Navigation", nav_open);
                
                // Handle navigation button clicks
                if self.nav_panel.should_go_next {
                    self.nav_panel.should_go_next = false;
                    if let Some(handle) = &self.handler {
                        self.trame_manager.continue_to_next_enabled(&mut self.data, handle.is_full_read());
                        // Parse ASN.1 to JSON for RRC messages
                        self.parse_current_rrc_message();
                    }
                }
                if self.nav_panel.should_go_previous {
                    self.nav_panel.should_go_previous = false;
                    // Call previous navigation method (need to expose it)
                    self.trame_manager.go_to_previous(&mut self.data);
                    // Parse ASN.1 to JSON for RRC messages
                    self.parse_current_rrc_message();
                }
                
                // Show other windows
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    if let Err(err) = one_window.show(ctx, &mut is_open, &mut self.data) {
                        log::error!("Error in window {}", one_window.name());
                        error_to_return.push(err);
                    }
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
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
    
    /// Parse ASN.1 from current RRC message and log as JSON
    fn parse_current_rrc_message(&self) {
        if let Some(trace) = self.data.get_current_trace() {
            if let Some(json) = trace.parse_asn1_to_json() {
                log::info!("RRC Message JSON:\n{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            }
        }
    }
}
