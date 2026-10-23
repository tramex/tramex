//! Frontend module
use crate::handlers::handler_file::FileHandler;
#[cfg(feature = "websocket")]
use crate::handlers::handler_ws::WsHandler;

#[cfg(feature = "websocket")]
use crate::event_system::WebSocketSource;
use crate::event_system::{Application, FileSource, PanelGroup, create_application_with_panels};
use crate::panels::navigation_panel::{NavState, NavigationPanel};
use crate::set_open;
use egui::{Ui, containers::menu::MenuButton, containers::menu::MenuConfig};
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

        // Start with no floating windows open: only the toolbar and the
        // Messages panel are shown. Persisted state restores later choices.
        Self {
            application,
            ..Default::default()
        }
    }

    /// Show the menu bar
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.source_set {
            MenuButton::new("Windows")
                .config(MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    let panel_groups = self.application.panel_groups();

                    ui.horizontal(|ui| {
                        if ui.button("Show all").clicked() {
                            for (_, name) in &panel_groups {
                                self.open_windows.insert((*name).to_owned());
                            }
                        }
                        if ui.button("Hide all").clicked() {
                            for (_, name) in &panel_groups {
                                self.open_windows.remove(*name);
                            }
                        }
                    });
                    ui.separator();

                    for (idx, group) in PanelGroup::WINDOWED.iter().enumerate() {
                        if idx > 0 {
                            ui.separator();
                        }
                        ui.label(egui::RichText::new(group.label()).strong());
                        for (_, panel_name) in panel_groups.iter().filter(|(g, _)| g == group) {
                            let mut is_open: bool = self.open_windows.contains(*panel_name);
                            ui.checkbox(&mut is_open, *panel_name);
                            set_open(&mut self.open_windows, panel_name, is_open);
                        }
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

        // Show collapse/expand button at the edge of the panel
        let button_text = if self.open_menu_connector { "◀" } else { "▶" };
        let button_tooltip = if self.open_menu_connector {
            "Hide side panel"
        } else {
            "Show side panel"
        };

        // Draw the toggle button on the left side
        let toggle_frame = egui::Frame::NONE.fill(ui.visuals().panel_fill);
        egui::Panel::left("panel_toggle")
            .resizable(false)
            .exact_size(20.0)
            .show_separator_line(false)
            .frame(toggle_frame)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    if ui
                        .add(egui::Button::new(button_text).frame(false))
                        .on_hover_text(button_tooltip)
                        .clicked()
                    {
                        self.open_menu_connector = !self.open_menu_connector;
                    }
                });
            });

        if self.open_menu_connector {
            egui::Panel::left("backend_panel")
                .resizable(false)
                .exact_size(240.0)
                .show_animated_inside(ui, self.open_menu_connector, |ui| {
                    ui.label(
                        egui::RichText::new("Source")
                            .small()
                            .color(crate::theme::ThemeColors::get(ui).text_weak),
                    );
                    ui.vertical_centered(|ui| {
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
                                            self.open_menu_connector = true;
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
                                                self.open_menu_connector = false;
                                                self.select_group(PanelGroup::Rrc);

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
                                    match ws_handler.show_ui(ui, self.source_set) {
                                        Ok(true) => {
                                            // Close requested
                                            self.connector = Some(Connector::WebSocket(WsHandler::new()));
                                            self.application.clear_data_source();
                                            self.source_set = false;
                                            self.open_menu_connector = true;
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
                                                self.open_menu_connector = false;
                                                self.select_group(PanelGroup::Rrc);
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
                    });
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

        if self.source_set {
            // Windows toggled by each group button (computed before borrowing self in the closures)
            let group_windows: Vec<(PanelGroup, Vec<&'static str>)> = PanelGroup::WINDOWED
                .iter()
                .rev()
                .map(|g| (*g, self.windows_for_group(*g)))
                .collect();

            // Build NavState from Application
            let nav_state = NavState {
                current_index: self.application.current_index(),
                event_count: self.application.event_count(),
                current_event: self.application.current_event(),
                total_count: self.application.total_event_count(),
                is_fully_loaded: self.application.is_fully_loaded(),
                ws_info: self.get_ws_info(),
            };

            let colors = crate::theme::ThemeColors::get(ui);

            // Group selected this frame (layers are applied after the toolbar is drawn)
            let mut selected_group: Option<PanelGroup> = None;

            // Fixed navigation toolbar with panel-group toggles on the right
            egui::Panel::top("nav_toolbar")
                .frame(
                    egui::Frame::new()
                        .fill(colors.bg_dark)
                        .inner_margin(egui::Margin::symmetric(8, 4)),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.nav_panel.show_toolbar(ui, &nav_state);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for (group, names) in &group_windows {
                                let all_open = names.iter().all(|n| self.open_windows.contains(*n));
                                let response = ui.selectable_label(all_open, group.label()).on_hover_text(names.join("\n"));
                                if response.clicked() {
                                    if all_open {
                                        for name in names {
                                            self.open_windows.remove(*name);
                                        }
                                    } else {
                                        selected_group = Some(*group);
                                    }
                                }
                            }
                        });
                    });
                });

            // Status bar: data source, loaded count, auto-loading state
            egui::Panel::bottom("status_bar")
                .frame(
                    egui::Frame::new()
                        .fill(colors.bg_dark)
                        .inner_margin(egui::Margin::symmetric(8, 2)),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(source_desc) = self.application.data_source_type() {
                            ui.label(egui::RichText::new(source_desc).small().color(colors.text_weak));
                            ui.separator();
                        }

                        let mut loaded = format!("Loaded {}", nav_state.event_count);
                        if let Some(total) = nav_state.total_count {
                            loaded.push_str(&format!(" / {}", total));
                        }
                        ui.label(egui::RichText::new(loaded).small().color(colors.text_weak));

                        if let Some((is_ws_available, is_auto_loading)) = nav_state.ws_info
                            && is_ws_available
                        {
                            ui.separator();
                            let (text, color) = if is_auto_loading {
                                ("● auto-loading", colors.success)
                            } else {
                                ("● paused", colors.warning)
                            };
                            ui.label(egui::RichText::new(text).small().color(color));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if nav_state.is_fully_loaded {
                                ui.label(egui::RichText::new("Fully loaded").small().color(colors.success));
                            }
                        });
                    });
                });

            if let Some(group) = selected_group {
                self.select_group(group);
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::same(8)))
            .show_inside(ui, |ui| {
                if self.source_set {
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

                    // Fixed central view: Messages panel (Main group)
                    self.application.show_main_panel(ui);

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

                    // Handle navigation requests from panels (e.g. clicking an arrow in the HARQ panel)
                    if let Some(target_index) = ui
                        .ctx()
                        .data_mut(|d| d.remove_temp::<usize>(egui::Id::new(crate::panels::NAVIGATE_REQUEST_ID)))
                    {
                        self.handle_goto_event(target_index, &mut error_to_return);
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
    ///
    /// Keeps requesting more data and retrying navigation until either a matching
    /// (enabled-layer) event is found, or the data source is exhausted / stops
    /// producing new events (e.g., WebSocket waiting for more data).
    fn handle_navigate_next(&mut self, errors: &mut Vec<TramexError>) {
        let mut navigated = self.application.navigate_next();
        log::debug!(
            "Navigation result: {}, current: {}, total: {}",
            navigated,
            self.application.current_index(),
            self.application.event_count()
        );

        // If at end, keep loading more data until we find a matching event or run out of data
        while !navigated && self.application.has_more_data() {
            let count_before = self.application.event_count();
            log::info!("Reached end of loaded events ({}), loading more...", count_before);
            if let Err(err) = self.application.request_more_data() {
                Self::collect_errors(errors, err);
                return;
            }
            // Process newly loaded events
            if let Err(err) = self.application.update() {
                Self::collect_errors(errors, err);
                return;
            }
            // Check if new events were actually loaded
            let count_after = self.application.event_count();
            if count_after > count_before {
                // New events loaded, try navigating again
                navigated = self.application.navigate_next();
            } else {
                // No new events were loaded (e.g., WebSocket waiting for data), stop here
                // The user can click Next again when more data arrives
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

    /// Select a panel group: open its windows, close the other groups' windows
    /// and enable the layers relevant to it
    fn select_group(&mut self, group: PanelGroup) {
        let names = self.windows_for_group(group);
        for (_, name) in self.application.panel_groups() {
            if names.contains(&name) {
                self.open_windows.insert(name.to_owned());
            } else {
                self.open_windows.remove(name);
            }
        }
        self.application.set_layers(layers_for_group(group));
    }

    /// Windows opened by a group shortcut: the group's own panels plus a few
    /// panels from other groups that are useful in that context
    fn windows_for_group(&self, group: PanelGroup) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self
            .application
            .panel_groups()
            .into_iter()
            .filter(|(g, _)| *g == group)
            .map(|(_, n)| n)
            .collect();
        if group == PanelGroup::Rrc {
            names.push("Chronograph RAN");
        }
        names
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
    pub fn show_layer_options(&mut self, ui: &mut egui::Ui) {
        let layers = self.application.layers_mut();

        egui::CollapsingHeader::new("Radio").default_open(true).show(ui, |ui| {
            layer_checkbox(ui, &mut layers.phy, "PHY");
            layer_checkbox(ui, &mut layers.mac, "MAC");
            layer_checkbox(ui, &mut layers.rlc, "RLC");
            layer_checkbox(ui, &mut layers.pdcp, "PDCP");
            layer_checkbox(ui, &mut layers.sdap, "SDAP");
            layer_checkbox(ui, &mut layers.rrc, "RRC");
            layer_checkbox(ui, &mut layers.nas, "NAS");
        });
        egui::CollapsingHeader::new("Core Network").default_open(true).show(ui, |ui| {
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
    }

    /// Show file-loading options for the File connector
    pub fn show_file_options(&mut self, ui: &mut egui::Ui) {
        if let Some(Connector::File(file_handler)) = &mut self.connector {
            file_handler.ui_options(ui);
        } else {
            ui.weak("File options are only available in File mode");
        }
    }
}

/// Layer filters to enable when a panel group is selected
fn layers_for_group(group: PanelGroup) -> tramex_tools::interface::layer::Layers {
    use tramex_tools::interface::layer::{LayerLogLevel, Layers};

    let mut layers = Layers::new();
    match group {
        PanelGroup::Rrc => layers.rrc = LayerLogLevel::Debug,
        PanelGroup::Nas => {
            layers.rrc = LayerLogLevel::Debug;
            layers.nas = LayerLogLevel::Debug;
            layers.ngap = LayerLogLevel::Debug;
            layers.gtpu = LayerLogLevel::Debug;
        }
        PanelGroup::Radio => layers.phy = LayerLogLevel::Debug,
        PanelGroup::Main => layers = Layers::new_optiniated(),
    }
    layers
}

/// Helper function to create a checkbox for LayerLogLevel
fn layer_checkbox(ui: &mut egui::Ui, layer: &mut tramex_tools::interface::layer::LayerLogLevel, text: &str) {
    use tramex_tools::interface::layer::LayerLogLevel;

    let mut checked = matches!(layer, LayerLogLevel::Debug);

    if ui.checkbox(&mut checked, text).changed() {
        *layer = if checked { LayerLogLevel::Debug } else { LayerLogLevel::Warn };
    }
}
