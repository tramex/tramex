//! ws handler panel
use eframe::egui;
use tramex_tools::{
    data::Data,
    errors::{ErrorCode, TramexError},
    interface::{interface_types::InterfaceTrait, layer::Layers, websocket::ws_connection::WsConnection},
    tramex_error,
};

use super::Handler;

/// Ws handler
pub struct WsHandler {
    /// Url Websocket
    pub url: String,

    /// WsConnection
    inner: Option<WsConnection>,
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
            url: "ws://127.0.0.1:9001".to_owned(),
            inner: None,
        }
    }

    /// Connect to a websocket
    /// # Errors
    /// Return an error if the connection failed
    pub fn connect(&mut self, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(), TramexError> {
        match WsConnection::connect(&self.url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.inner = Some(WsConnection::new(ws_sender, ws_receiver));
                Ok(())
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", &self.url, error);
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
    /// # Errors
    /// Return an error if the closing failed
    fn close_ws(&mut self) -> Result<(), TramexError> {
        if let Some(interface_ws) = &mut self.inner {
            return interface_ws.close_impl();
        }
        Ok(())
    }
}

impl Handler for WsHandler {
    fn ui_options(&mut self, ui: &mut egui::Ui) {
        ui.label("Websocket Options");
        if let Some(interface_ws) = &mut self.inner {
            ui.horizontal(|ui| {
                ui.label("Max incoming frame size: ");
                ui.add(
                    egui::DragValue::new(&mut interface_ws.asking_size_max)
                        .speed(2.0)
                        .range(64.0..=4096.0),
                );
            });
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _data: &mut Data, new_ctx: egui::Context) -> Result<bool, TramexError> {
        if self.inner.is_some() {
            self.display_url(ui, false);
            if let Some(interface_ws) = &mut self.inner {
                if interface_ws.connecting {
                    ui.label("Connecting...");
                    ui.spinner();
                } else {
                    ui.label(format!("Name: {}", &interface_ws.name));
                    if ui.button("Close").clicked() {
                        match self.close_ws() {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            if (self.display_url(ui, true) && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.button("Connect").clicked()
            {
                let wakeup_fn = move || new_ctx.request_repaint(); // wake up UI thread on new message
                match self.connect(wakeup_fn) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
            }

            Ok(false)
        }
    }

    fn close(&mut self) -> Result<(), TramexError> {
        self.close_ws()
    }

    fn try_recv(&mut self, data: &mut Data) -> Result<(), Vec<TramexError>> {
        if let Some(interface_ws) = &mut self.inner {
            return interface_ws.try_recv(data);
        }
        Ok(())
    }

    fn get_more_data(&mut self, layer_list: Layers, data: &mut Data) -> Result<(), Vec<TramexError>> {
        if let Some(interface_ws) = &mut self.inner {
            return interface_ws.get_more_data(layer_list, data);
        }
        Ok(())
    }

    fn show_available(&self, ui: &mut egui::Ui) {
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

    fn is_full_read(&self) -> bool {
        false
    }

    fn is_interface(&self) -> bool {
        self.inner.is_some()
    }

    fn is_interface_available(&self) -> bool {
        if let Some(interface_ws) = &self.inner {
            return interface_ws.available;
        }
        false
    }
}
