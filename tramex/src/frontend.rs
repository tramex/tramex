//! Frontend module
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;

#[cfg(feature = "websocket")]
use crate::event_system::WebSocketSource;
use crate::event_system::{Application, FileSource, create_application_with_panels};
use crate::panels::navigation_panel::{NavState, NavigationPanel};
use crate::set_open;
use egui::Ui;
use std::collections::BTreeSet;
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

/// Connector state — holds the UI-only handler for the active mode
#[allow(clippy::large_enum_variant)]
enum Connector {
    /// File handler
    File(FileHandler),
    /// WebSocker handler
    #[cfg(feature = "websocket")]
    WebSocket(WsHandler),
}

#[derive(serde::Deserialize, serde::Serialize)]
/// FrontEnd struct
pub struct FrontEnd {
    /// Open windows
    pub open_windows: BTreeSet<String>,

    /// Open menu connector
    pub open_menu_connector: bool,

    /// Radio choice
    pub radio_choice: Choice,

    #[serde(skip)]
    /// Connector UI (file picker or WS connector)
    connector: Option<Connector>,

    #[serde(skip)]
    /// Navigation panel
    nav_panel: NavigationPanel,

    #[serde(skip)]
    /// Event-driven application controller
    application: Application,

    #[serde(skip)]
    /// Whether a data source has been set on Application for the current connector
    source_set: bool,
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            open_windows: BTreeSet::new(),
            open_menu_connector: true,
            radio_choice: Choice::default(),
            connector: None,
            nav_panel: NavigationPanel::new(),
            application: Application::new(),
            source_set: false,
        }
    }
}

impl FrontEnd {
    /// Create a new frontend
    pub fn new() -> Self {
        log::info!("FrontEnd: Initializing event system");
        let application = create_application_with_panels();

        let mut open_windows = BTreeSet::new();
        for panel_name in application.panel_names() {
            open_windows.insert(panel_name.to_owned());
        }
        open_windows.insert("Navigation".to_owned());

        Self {
            open_windows,
            application,
            ..Default::default()
        }
    }

    /// Show the menu bar
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.source_set {
            ui.menu_button("Windows", |ui| {
                let mut nav_open = self.open_windows.contains("Navigation");
                ui.checkbox(&mut nav_open, "Navigation");
                set_open(&mut self.open_windows, "Navigation", nav_open);

                for panel_name in self.application.panel_names() {
                    let mut is_open: bool = self.open_windows.contains(panel_name);
                    ui.checkbox(&mut is_open, panel_name);
                    set_open(&mut self.open_windows, panel_name, is_open);
                }
            });
        }
    }

    /// Check if interface is available (source set on Application)
    pub fn interface_available(&self) -> bool {
        self.source_set
    }

    /// Show the UI connector
    /// # Errors
    /// Return a vector of TramexError
    pub fn ui_connector(&mut self, ui: &mut egui::Ui) -> Result<(), Vec<TramexError>> {
        let mut errors = vec![];
        if self.open_menu_connector {
            egui::Panel::left("backend_panel")
                .resizable(false)
                .exact_size(240.0)
                .show_animated_inside(ui, self.open_menu_connector, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Connector");
                        let save = self.radio_choice.clone();
                        ui.vertical(|ui| {
                            let enabled = !self.source_set;
                            ui.add_enabled_ui(enabled, |ui| {
                                ui.label("Choose ws or file");
                                ui.radio_value(&mut self.radio_choice, Choice::File, "File");
                                #[cfg(feature = "websocket")]
                                ui.radio_value(&mut self.radio_choice, Choice::WebSocket, "WebSocket");
                            });
                        });

                        // Create connector if needed
                        match &self.radio_choice {
                            #[cfg(feature = "websocket")]
                            Choice::WebSocket => {
                                if save != self.radio_choice || self.connector.is_none() {
                                    self.connector = Some(Connector::WebSocket(WsHandler::new()));
                                }
                            }
                            Choice::File => {
                                if save != self.radio_choice || self.connector.is_none() {
                                    self.connector = Some(Connector::File(FileHandler::new()));
                                }
                            }
                        };

                        // Show connector UI
                        ui.vertical(|ui| {
                            match &mut self.connector {
                                Some(Connector::File(file_handler)) => {
                                    match file_handler.show_ui(ui) {
                                        Ok(true) => {
                                            // Close requested
                                            self.connector = Some(Connector::File(FileHandler::new()));
                                            self.application.clear_all();
                                            self.source_set = false;
                                        }
                                        Ok(false) => {
                                            // Check if file became available and source not yet set
                                            if !self.source_set
                                                && file_handler.is_available()
                                                && let Some(file) = file_handler.take_file()
                                            {
                                                log::info!("File loaded — creating FileSource");
                                                let source = FileSource::from_file(file);
                                                self.application.set_data_source(Box::new(source));
                                                self.source_set = true;

                                                // Load first batch
                                                if let Err(err) = self.application.request_more_data() {
                                                    Self::collect_errors(&mut errors, err);
                                                }
                                                // Process the loaded events
                                                if let Err(err) = self.application.update() {
                                                    Self::collect_errors(&mut errors, err);
                                                }
                                                // Metadata sync happens inside Application.update()
                                            }
                                        }
                                        Err(err) => {
                                            errors.push(err);
                                        }
                                    }
                                }
                                #[cfg(feature = "websocket")]
                                Some(Connector::WebSocket(ws_handler)) => {
                                    match ws_handler.show_ui(ui) {
                                        Ok(true) => {
                                            // Close requested
                                            self.connector = Some(Connector::WebSocket(WsHandler::new()));
                                            self.application.clear_all();
                                            self.source_set = false;
                                        }
                                        Ok(false) => {
                                            // Check if WS connected and source not yet set
                                            if !self.source_set
                                                && ws_handler.is_available()
                                                && let Some(connection) = ws_handler.take_connection()
                                            {
                                                log::info!("WebSocket connected — creating WebSocketSource");
                                                let url = ws_handler.url.clone();
                                                let source = WebSocketSource::new(url, connection);
                                                self.application.set_data_source(Box::new(source));
                                                self.source_set = true;
                                            }
                                        }
                                        Err(err) => {
                                            errors.push(err);
                                        }
                                    }
                                }
                                None => {}
                            }
                        });

                        // Show file options only in File mode
                        if matches!(self.radio_choice, Choice::File)
                            && let Some(Connector::File(file_handler)) = &mut self.connector
                        {
                            file_handler.ui_options(ui);
                        }
                    });
                    ui.separator();

                    // Show layer options
                    self.show_layer_options(ui);
                });
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }

    #[cfg(feature = "ai")]
    /// Forward AI config to all subscriber panels
    pub fn set_ai_config(&mut self, key: &str, provider: &tramex_tools::ai::AIProvider, model: &str) {
        self.application.set_ai_config(key, provider, model);
    }

    /// Show the UI
    /// # Errors
    /// Return a vector of TramexError
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Result<(), Vec<TramexError>> {
        let mut error_to_return = vec![];

        // Poll data source and process new events
        if let Err(err) = self.application.update() {
            Self::collect_errors(&mut error_to_return, err);
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.source_set {
                // Build NavState from Application
                let nav_state = NavState {
                    current_index: self.application.current_index(),
                    event_count: self.application.event_count(),
                    current_event: self.application.current_event(),
                    total_count: self.application.total_event_count(),
                    is_fully_loaded: self.application.is_fully_loaded(),
                    ws_info: self.get_ws_info(),
                };

                // Show navigation panel
                let mut nav_open = self.open_windows.contains("Navigation");
                if let Err(err) = self.nav_panel.show_nav(ui, &mut nav_open, &nav_state) {
                    log::error!("Error in Navigation panel");
                    error_to_return.push(err);
                }
                set_open(&mut self.open_windows, "Navigation", nav_open);

                // Handle navigation
                if self.nav_panel.should_go_next {
                    self.nav_panel.should_go_next = false;
                    self.handle_navigate_next(&mut error_to_return);
                }
                if self.nav_panel.should_go_previous {
                    self.nav_panel.should_go_previous = false;
                    self.application.navigate_previous();
                }

                // Handle goto event
                if let Some(target_index) = self.nav_panel.should_goto_event.take() {
                    self.handle_goto_event(target_index, &mut error_to_return);
                }

                // Handle WebSocket auto-loading toggle
                if self.nav_panel.should_toggle_auto_loading {
                    self.nav_panel.should_toggle_auto_loading = false;
                    self.application.toggle_auto_loading();
                }

                // Show panel windows and sync open state (X button)
                let panel_results = self.application.show_panel_windows(ui, &self.open_windows);
                for (panel_name, is_open, result) in panel_results {
                    if is_open {
                        self.open_windows.insert(panel_name.clone());
                    } else {
                        self.open_windows.remove(&panel_name);
                    }
                    if let Err(err) = result {
                        log::error!("Error showing panel: {:?}", err);
                        error_to_return.push(err);
                    }
                }
            } else {
                // Show connector status
                match &self.connector {
                    Some(Connector::File(fh)) => fh.show_status(ui),
                    #[cfg(feature = "websocket")]
                    Some(Connector::WebSocket(wh)) => wh.show_status(ui),
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

    /// Handle navigate-next with on-demand loading
    fn handle_navigate_next(&mut self, errors: &mut Vec<TramexError>) {
        let mut navigated = self.application.navigate_next();
        log::debug!(
            "Navigation result: {}, current: {}, total: {}",
            navigated,
            self.application.current_index(),
            self.application.event_count()
        );

        // If at end, keep loading batches until we find an enabled event
        while !navigated {
            if self.application.has_more_data() {
                log::info!(
                    "Reached end of loaded events ({}), loading more...",
                    self.application.event_count()
                );
                if let Err(err) = self.application.request_more_data() {
                    Self::collect_errors(errors, err);
                    break;
                }
                // Process newly loaded events
                if let Err(err) = self.application.update() {
                    Self::collect_errors(errors, err);
                    break;
                }
                // Check if new events were actually loaded
                if self.application.event_count() == 0 {
                    break;
                }
                navigated = self.application.navigate_next();
            } else {
                break;
            }
        }
    }

    /// Handle goto-event: load data until target index is available, then navigate
    fn handle_goto_event(&mut self, target_index: usize, errors: &mut Vec<TramexError>) {
        log::info!(
            "Goto event: target index {} (loaded: {})",
            target_index,
            self.application.event_count()
        );

        // Load batches until we have enough events
        while self.application.event_count() <= target_index {
            if !self.application.has_more_data() {
                log::warn!(
                    "Goto event: cannot reach index {}, source exhausted at {} events",
                    target_index,
                    self.application.event_count()
                );
                break;
            }
            if let Err(err) = self.application.request_more_data() {
                Self::collect_errors(errors, err);
                break;
            }
            if let Err(err) = self.application.update() {
                Self::collect_errors(errors, err);
                break;
            }
        }

        // Navigate to the target
        if target_index < self.application.event_count() {
            self.application.navigate_to(target_index);
        }
    }

    /// Get WebSocket info for NavigationPanel
    fn get_ws_info(&self) -> Option<(bool, bool)> {
        // Check if the connector is a WebSocket and source is set
        #[cfg(feature = "websocket")]
        if self.source_set
            && let Some(Connector::WebSocket(_)) = &self.connector
        {
            return Some((true, self.application.is_auto_loading()));
        }
        None
    }

    /// Collect errors, filtering out non-critical ones
    fn collect_errors(errors: &mut Vec<TramexError>, new_errors: Vec<TramexError>) {
        for err in new_errors {
            if !matches!(err.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                errors.push(err);
            }
        }
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
        *layer = if checked { LayerLogLevel::Debug } else { LayerLogLevel::Warn };
    }
}
