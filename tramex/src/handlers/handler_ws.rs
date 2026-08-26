//! WebSocket handler panel — UI-only connector for WebSocket
use eframe::egui;
use tramex_tools::{
    data::Data,
    errors::{ErrorCode, TramexError},
    interface::websocket::ws_connection::WsConnection,
    tramex_error,
};

/// WebSocket handler — UI for connecting to WebSocket
pub struct WsHandler {
    /// Url Websocket
    pub url: String,

    /// WsConnection (held temporarily until consumed by Application)
    inner: Option<WsConnection>,

    /// Last error message from connection attempt
    last_error: Option<String>,
}

impl Default for WsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHandler {
    /// Create a new ws handler
    pub fn new() -> Self {
        Self {
            url: "ws://137.194.194.35:9001".to_owned(),
            inner: None,
            last_error: None,
        }
    }

    /// Connect to a websocket
    /// # Errors
    /// Return an error if the connection failed
    pub fn connect(&mut self, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(), TramexError> {
        self.last_error = None;
        log::info!("🔌 WsHandler::connect() - attempting to connect to {}", &self.url);
        match WsConnection::connect(&self.url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                log::info!("🔌 WsHandler::connect() - WsConnection::connect succeeded, creating WsConnection");
                self.inner = Some(WsConnection::new(ws_sender, ws_receiver));
                Ok(())
            }
            Err(error) => {
                log::error!("❌ WsHandler::connect() - Failed to connect to {:?}: {}", &self.url, error);
                Err(tramex_error!(error.to_string(), ErrorCode::WebSocketFailedToConnect))
            }
        }
    }

    /// Display the url
    pub fn display_url(&mut self, ui: &mut egui::Ui, enabled: bool) -> bool {
        let mut lost_focus = false;
        ui.add_enabled_ui(enabled, |ui_enabled| {
            ui_enabled.label("URL:");
            lost_focus = ui_enabled.text_edit_singleline(&mut self.url).lost_focus();
        });
        lost_focus
    }

    /// Close the websocket
    ///
    /// # Errors
    ///
    /// Fails when WS fails to close
    fn close_ws(&mut self) -> Result<(), TramexError> {
        if let Some(interface_ws) = &mut self.inner {
            return interface_ws.close_impl();
        }
        Ok(())
    }

    /// Show WebSocket UI.
    /// Returns Ok(true) if close was requested, Ok(false) otherwise.
    ///
    /// # Errors
    ///
    pub fn show_ui(&mut self, ui: &mut egui::Ui, source_connected: bool) -> Result<bool, TramexError> {
        if source_connected {
            self.display_url(ui, false);
            ui.label(egui::RichText::new("✓ Connected").color(egui::Color32::GREEN));
            return Ok(ui.button("Disconnect").clicked());
        }

        if let Some(interface_ws) = &mut self.inner {
            let mut handshake_data = Data::default();
            if let Err(errors) = interface_ws.try_recv(&mut handshake_data) {
                self.last_error = Some(
                    errors
                        .iter()
                        .map(|error| error.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; "),
                );
            }
        }

        if self.inner.is_some() {
            self.display_url(ui, false);
            if let Some(interface_ws) = &mut self.inner {
                // Log current state for debugging
                log::trace!(
                    "🖥️ WsHandler::show_ui() - inner.is_some()=true, connecting={}, available={}",
                    interface_ws.connecting,
                    interface_ws.available
                );

                // Always show connection status
                ui.horizontal(|ui| {
                    if interface_ws.connecting {
                        ui.label(egui::RichText::new("⏳ Connecting").color(egui::Color32::ORANGE));
                        ui.spinner();
                    } else if interface_ws.available {
                        ui.label(egui::RichText::new("✓ Connected").color(egui::Color32::GREEN));
                    } else {
                        ui.label(egui::RichText::new("✗ Disconnected").color(egui::Color32::RED));
                    }
                });

                if !interface_ws.connecting {
                    if !interface_ws.name.is_empty() {
                        ui.label(format!("Server: {}", &interface_ws.name));
                    }
                    if let Some(error) = &self.last_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                    if ui.button("Close").clicked() {
                        self.close_ws()?;
                        self.inner = None;
                        self.last_error = None;
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            // Show "Not connected" status
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("○ Not connected").color(egui::Color32::GRAY));
            });

            let url_lost_focus = self.display_url(ui, true);
            let connect_clicked = ui.button("Connect").clicked();
            let enter_pressed = url_lost_focus && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if connect_clicked || enter_pressed {
                let ctx = ui.ctx().clone();
                let wakeup_fn = move || ctx.request_repaint();
                self.connect(wakeup_fn)?;
            }
            Ok(false)
        }
    }

    /// Take the connection out (consumed once to create a WebSocketSource)
    pub fn take_connection(&mut self) -> Option<WsConnection> {
        self.inner.take()
    }

    /// Check if a connection is established and ready
    pub fn is_available(&self) -> bool {
        self.inner.as_ref().is_some_and(|ws| ws.available)
    }

    /// Check if connecting
    pub fn is_connecting(&self) -> bool {
        self.inner.as_ref().is_some_and(|ws| ws.connecting)
    }

    /// Check if handler has an active connection
    pub fn is_active(&self) -> bool {
        self.inner.is_some()
    }

    /// Show status when not yet available
    pub fn show_status(&self, ui: &mut egui::Ui) {
        if let Some(interface_ws) = &self.inner {
            if interface_ws.connecting {
                ui.label("Websocket connecting...");
                ui.spinner();
                return;
            }
            if interface_ws.available {
                ui.label("Websocket connected");
                return;
            }
        }
        ui.label("Websocket Not connected");
    }
}
