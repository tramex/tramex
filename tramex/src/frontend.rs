//! Frontend module
use crate::handlers::Handler;
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;

use crate::panels::{
    navigation_panel::NavigationPanel,
};
use crate::event_system::{Application, create_application_with_panels};
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

    /// Open menu connector
    pub open_menu_connector: bool,

    #[serde(skip)]
    /// File upload
    handler: Option<Box<dyn Handler>>,


    /// Radio choice
    pub radio_choice: Choice,
    
    #[serde(skip)]
    /// Navigation panel (separate reference for button handling)
    nav_panel: NavigationPanel,
    
    #[serde(skip)]
    /// Event-driven application controller
    application: Application,
    
    #[serde(skip)]
    /// Last count of events transferred to Application (to avoid duplicates)
    last_transferred_count: usize,
    
    #[serde(skip)]
    /// Track if we've done initial data load for current file
    initial_load_done: bool,
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            data: Data::default(),
            open_windows: BTreeSet::new(),
            open_menu_connector: true,
            radio_choice: Choice::default(),
            handler: None,
            nav_panel: NavigationPanel::new(),
            application: Application::new(), // Will be properly initialized in new()
            last_transferred_count: 0,
            initial_load_done: false,
        }
    }
}

impl FrontEnd {
    /// Create a new frontend
    pub fn new() -> Self {
        // Initialize event system with all panels
        log::info!("FrontEnd: Initializing event system");
        let application = create_application_with_panels();
        
        // Build open_windows set from Application's panels
        let mut open_windows = BTreeSet::new();
        for panel_name in application.panel_names() {
            open_windows.insert(panel_name.to_owned());
        }
        // Add Navigation panel to open windows by default
        open_windows.insert("Navigation".to_owned());
        
        Self {
            open_windows,
            application,
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
                
                // Panels from Application
                for panel_name in self.application.panel_names() {
                    let mut is_open: bool = self.open_windows.contains(panel_name);
                    ui.checkbox(&mut is_open, panel_name);
                    set_open(&mut self.open_windows, panel_name, is_open);
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
                                        // No longer need to clear old panel instances
                                        // Reset transfer counter when clearing
                                        self.last_transferred_count = 0;
                                        self.initial_load_done = false;  // Reset for next file
                                        
                                        // Clear Application (which notifies all panels)
                                        self.application.clear_all();
                                    }
                                    Ok(false) => {
                                        // Transfer new events from Data to Application
                                        let current_count = self.data.events.len();
                                        if current_count > self.last_transferred_count {
                                            // Transfer only new events
                                            let new_events: Vec<_> = self.data.events[self.last_transferred_count..].to_vec();
                                            let batch_size = new_events.len();
                                            self.application.add_events(new_events);
                                            log::info!("Transferred {} new events to Application (total: {})", 
                                                batch_size, current_count);
                                            self.last_transferred_count = current_count;
                                        }
                                    }
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
                    
                    // Show layer options
                    self.show_layer_options(ui);
                    
                    if self.interface_available() {
                        if let Some(handle) = &mut self.handler {
                            // Trigger initial data load when file first becomes available
                            if !self.initial_load_done && self.data.events.is_empty() {
                                log::info!("File available - triggering initial data load");
                                // Load first batch of events
                                if let Err(err) = handle.get_more_data(self.application.layers().clone(), &mut self.data) {
                                    for one_error in err {
                                        if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                                            errors.push(one_error);
                                        }
                                    }
                                }
                                self.initial_load_done = true;
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
        
        // Try to receive any incoming WebSocket messages or load file data
        let previous_count = self.data.events.len();
        
        // Always load data into self.data first (legacy system)
        if let Some(handle) = &mut self.handler {
            if let Err(errors_vect) = handle.try_recv(&mut self.data) {
                for one_error in errors_vect {
                    if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                        error_to_return.push(one_error);
                    }
                }
            }
        }
        
        // Sync metadata from Data to Application
        self.application.sync_metadata_from_data(&self.data);
        
        // Transfer new events from Data to Application
        let current_count = self.data.events.len();
        // log::debug!("{}, {}", current_count, self.last_transferred_count);
        if current_count > self.last_transferred_count {
            // Transfer only new events
            let new_events: Vec<_> = self.data.events[self.last_transferred_count..].to_vec();
            let batch_size = new_events.len();
            self.application.add_events(new_events);
            log::debug!("UI loop: Transferred {} new events to Application (total: {})", 
                batch_size, current_count);
            self.last_transferred_count = current_count;
        }
        
        // Call Application::update() for any data source polling
        if let Err(errors_vect) = self.application.update() {
            for one_error in errors_vect {
                if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                    error_to_return.push(one_error);
                }
            }
        }
        
        // Check if new events arrived during auto-loading
        let should_navigate = if let Some(handle) = &self.handler {
            handle.is_websocket() && handle.is_ws_auto_loading() && current_count > previous_count
        } else {
            false
        };
        
        if should_navigate {
            // New events received during auto-loading - navigate to last event
            let new_events_count = current_count - previous_count;
            log::debug!("Auto-loading: {} new events received, navigating to last", new_events_count);
            
            // Navigate to the last event
            if current_count > 0 {
                self.application.navigate_to(current_count - 1);
                self.data.current_index = self.application.current_index();
            }
        }
        
        // For WebSocket: check if we should send a new request (after receiving previous response)
        if let Some(handle) = &mut self.handler {
            if handle.is_websocket() && handle.should_ws_request_more() {
                if let Err(errors_vect) = handle.get_more_data(self.application.layers().clone(), &mut self.data) {
                    for one_error in errors_vect {
                        if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                            error_to_return.push(one_error);
                        }
                    }
                }
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.interface_available() {
                // Update total count and is_full_read in navigation panel
                // Use total file size if known (for files), otherwise use loaded count
                self.nav_panel.total_count = self.handler.as_ref()
                    .and_then(|h| h.get_total_event_count())
                    .or(Some(self.application.event_count()));
                // Use legacy handler's is_full_read since we're still using legacy file loading
                self.nav_panel.is_full_read = self.handler.as_ref()
                    .map(|h| h.is_full_read())
                    .unwrap_or(true);
                
                // Show navigation panel with WebSocket info if applicable
                let mut nav_open = self.open_windows.contains("Navigation");
                let ws_info = self.handler.as_ref().and_then(|h| {
                    if h.is_websocket() && h.is_interface_available() {
                        Some((true, h.is_ws_auto_loading()))
                    } else {
                        None
                    }
                });
                if let Err(err) = self.nav_panel.show_with_ws_info(
                    ctx, 
                    &mut nav_open, 
                    &mut self.data,
                    ws_info
                ) {
                    log::error!("Error in Navigation panel");
                    error_to_return.push(err);
                }
                set_open(&mut self.open_windows, "Navigation", nav_open);
                
                // Handle navigation button clicks
                if self.nav_panel.should_go_next {
                    self.nav_panel.should_go_next = false;
                    
                    // Try to navigate
                    let mut navigated = self.application.navigate_next();
                    log::debug!("Navigation result: {}, current index: {}, total events: {}", 
                        navigated, self.application.current_index(), self.application.event_count());
                    
                    // If we couldn't navigate (reached end), keep loading batches until we find an enabled event
                    if !navigated {
                        log::debug!("Failed to navigate, will try loading more batches");
                    }
                    while !navigated {
                        if let Some(handle) = &mut self.handler {
                            if !handle.is_full_read() {
                                log::info!("Reached end of loaded events ({}), loading more...", self.application.event_count());
                                
                                // Load next batch
                                if let Err(err) = handle.get_more_data(self.application.layers().clone(), &mut self.data) {
                                    for one_error in err {
                                        if !matches!(one_error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                                            error_to_return.push(one_error);
                                        }
                                    }
                                    break; // Stop on error
                                } else {
                                    // Transfer new events to Application
                                    let current_count = self.data.events.len();
                                    if current_count > self.last_transferred_count {
                                        let new_events: Vec<_> = self.data.events[self.last_transferred_count..].to_vec();
                                        let batch_size = new_events.len();
                                        self.application.add_events(new_events);
                                        log::debug!("Loaded {} more events", batch_size);
                                        self.last_transferred_count = current_count;
                                        
                                        // Try navigating again
                                        navigated = self.application.navigate_next();
                                        // Loop continues if still not navigated
                                    } else {
                                        break; // No new events loaded
                                    }
                                }
                            } else {
                                break; // File fully read
                            }
                        } else {
                            break; // No handler
                        }
                    }
                    
                    // Sync legacy data index with Application for panels that still use PanelController::show()
                    self.data.current_index = self.application.current_index();
                }
                if self.nav_panel.should_go_previous {
                    self.nav_panel.should_go_previous = false;
                    self.application.navigate_previous();
                    // Sync legacy data index with Application for panels that still use PanelController::show()
                    self.data.current_index = self.application.current_index();
                }
                
                // Handle WebSocket auto-loading toggle
                if self.nav_panel.should_toggle_auto_loading {
                    self.nav_panel.should_toggle_auto_loading = false;
                    if let Some(handle) = &mut self.handler {
                        handle.toggle_ws_auto_loading();
                    }
                }
                
                // Show panel windows through Application's EventSubscriber system
                let panel_errors = self.application.show_panel_windows(ctx, &self.open_windows);
                for (_panel_name, result) in panel_errors {
                    if let Err(err) = result {
                        log::error!("Error showing panel: {:?}", err);
                        error_to_return.push(err);
                    }
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
    
    /// Show layer filtering options
    fn show_layer_options(&mut self, ui: &mut egui::Ui) {
        let layers = self.application.layers_mut();
        
        ui.collapsing("Layers", |ui| {
            ui.collapsing("Radio", |ui| {
                layer_checkbox(ui, &mut layers.phy, "PHY");
                layer_checkbox(ui, &mut layers.mac, "MAC");
                layer_checkbox(ui, &mut layers.rlc, "RLC");
                layer_checkbox(ui, &mut layers.pdcp, "PDCP");
                layer_checkbox(ui, &mut layers.sdap, "SDAP");
                layer_checkbox(ui, &mut layers.rrc, "RRC");
                layer_checkbox(ui, &mut layers.nas, "NAS");
            });
            ui.collapsing("Core Network", |ui| {
                layer_checkbox(ui, &mut layers.s72, "S72");
                layer_checkbox(ui, &mut layers.s1ap, "S1AP");
                layer_checkbox(ui, &mut layers.ngap, "NGAP");
                layer_checkbox(ui, &mut layers.gtpu, "GTPU");
                layer_checkbox(ui, &mut layers.x2ap, "X2AP");
                layer_checkbox(ui, &mut layers.xnap, "XnAP");
                layer_checkbox(ui, &mut layers.m2ap, "M2AP");
                layer_checkbox(ui, &mut layers.lppa, "LPPa");
                layer_checkbox(ui, &mut layers.nrppa, "NRPPa");
                layer_checkbox(ui, &mut layers.trx, "TRX");
            });
        });
    }
}

/// Helper function to create a checkbox for LayerLogLevel
fn layer_checkbox(ui: &mut egui::Ui, layer: &mut tramex_tools::interface::layer::LayerLogLevel, text: &str) {
    use tramex_tools::interface::layer::LayerLogLevel;
    
    let mut checked = matches!(layer, LayerLogLevel::Debug);
    
    if ui.checkbox(&mut checked, text).changed() {
        *layer = if checked {
            LayerLogLevel::Debug
        } else {
            LayerLogLevel::Warn
        };
    }
}
