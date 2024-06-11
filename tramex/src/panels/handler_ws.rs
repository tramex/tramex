//! ws handler panel
use eframe::egui;
use tramex_tools::{connector::Connector, errors::TramexError, interface::interface_types::Interface};

use super::Handler;

#[derive(serde::Deserialize, serde::Serialize, Default)]
/// Ws handler
pub struct WsHandler {
    /// Url Websocket
    pub url: String,
}

impl WsHandler {
    /// Create a new ws handler
    pub fn new() -> Self {
        Self {
            url: "ws://127.0.0.1:9001".to_owned(),
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
}

impl Handler for WsHandler {
    fn ui_options(&mut self, ui: &mut egui::Ui) {
        ui.label("Websocket Options");
    }

    fn ui(&mut self, ui: &mut egui::Ui, conn: &mut Connector, new_ctx: egui::Context) -> Result<(), TramexError> {
        match &conn.interface {
            Some(Interface::Ws(interface_ws)) => {
                self.display_url(ui, false);
                if interface_ws.connecting {
                    ui.label("Connecting...");
                    ui.spinner();
                } else if ui.button("Close").clicked() {
                    if let Some(Interface::Ws(interface_ws)) = &mut conn.interface {
                        match interface_ws.close_impl() {
                            Ok(_) => {
                                conn.clear_interface();
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    };
                }
                Ok(())
            }
            _ => {
                if (self.display_url(ui, true) && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Connect").clicked()
                {
                    let wakeup_fn = move || new_ctx.request_repaint(); // wake up UI thread on new message
                    return conn.connect(&self.url, wakeup_fn);
                }

                Ok(())
            }
        }
    }
}
