use eframe::egui::{self};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeSet};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use crate::panels::{
    AboutPanel, FileHandler, LogicalChannels, MessageBox, PanelController, SocketManager,
};
use crate::{Data, WebSocketLog};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ExampleApp {
    pub url: String,
    #[serde(skip)]
    pub error: String,
    #[serde(skip)]
    frontend: Option<FrontEnd>,
    file_upload: Option<FileHandler>,
}

impl ExampleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl Default for ExampleApp {
    fn default() -> Self {
        Self {
            url: "ws://137.194.194.51:9001".to_owned(),
            error: Default::default(),
            frontend: None,
            file_upload: None,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                ui.menu_button("File", |ui| {
                    if ui.button("Upload a file").clicked() {
                        // TODO open file dialog
                        if self.file_upload.is_none() {
                            self.file_upload = Some(FileHandler::new());
                        } else {
                            self.file_upload = None;
                        }
                    }
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                if let Some(current_frontend) = &mut self.frontend {
                    if current_frontend.connected {
                        ui.menu_button("Windows", |ui| {
                            for one_window in current_frontend.windows.iter_mut() {
                                let mut is_open: bool = current_frontend
                                    .data
                                    .borrow()
                                    .open_windows
                                    .contains(one_window.name());
                                ui.checkbox(&mut is_open, one_window.name());
                                set_open(
                                    &mut current_frontend.data.borrow_mut().open_windows,
                                    one_window.name(),
                                    is_open,
                                );
                            }
                        });
                    }
                }
            });
        });

        egui::TopBottomPanel::top("server").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                if self.frontend.is_some() {
                    ui.label(&self.url);
                    if ui.button("Close").clicked() {
                        // TODO close connection
                        self.frontend = None;
                    }
                } else {
                    if (ui.text_edit_singleline(&mut self.url).lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Connect").clicked()
                    {
                        self.connect(ctx.clone());
                    }
                }
            });
        });

        if !self.error.is_empty() {
            egui::TopBottomPanel::top("error").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Error:");
                    ui.colored_label(egui::Color32::RED, &self.error);
                });
            });
        }

        if let Some(frontend) = &mut self.frontend {
            frontend.ui(ctx);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.horizontal(|ui| ui.vertical(|_ui| {})));
        }
        if let Some(fu) = &mut self.file_upload {
            fu.show(ctx, &mut true);
            if let Ok(result) = fu.get_result() {
                log::info!("File upload result: {:?}", result);
                // create fake websocket handler
                // self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.file_upload = None;
            }
        }
    }
}

impl ExampleApp {
    fn connect(&mut self, ctx: egui::Context) {
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        let options = ewebsock::Options {
            max_incoming_frame_size: 500,
        };
        match ewebsock::connect_with_wakeup(&self.url, options, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.error.clear();
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", &self.url, error);
                self.error = error;
            }
        }
    }
}

struct FrontEnd {
    ws_receiver: Rc<RefCell<WsReceiver>>,
    pub windows: Vec<Box<dyn PanelController>>,
    pub data: Rc<RefCell<Data>>,
    pub connected: bool,
    pub error: bool,
    pub error_str: String,
}

fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

impl FrontEnd {
    fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        let ref_ws_receiver = Rc::new(RefCell::new(ws_receiver));

        let data = Data {
            ws_sender: ws_sender,
            events: Vec::new(),
            current_index: 0,
            open_windows: BTreeSet::new(),
        };
        let ref_data = Rc::new(RefCell::new(data));
        let mb = MessageBox::new(Rc::clone(&ref_data));
        let sm = SocketManager::new(Rc::clone(&ref_data));
        let lc = LogicalChannels::new(Rc::clone(&ref_data));
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<AboutPanel>::default(),
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<SocketManager>::new(sm),
        ];
        for one_box in wins.iter() {
            ref_data
                .borrow_mut()
                .open_windows
                .insert(one_box.name().to_owned());
        }
        Self {
            data: ref_data,
            ws_receiver: ref_ws_receiver,
            windows: wins,
            connected: false,
            error: false,
            error_str: "".to_string(),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        while let Some(event) = self.ws_receiver.borrow_mut().try_recv() {
            match event {
                WsEvent::Message(msg) => match msg {
                    WsMessage::Text(event_text) => {
                        let decoded: Result<WebSocketLog, serde_json::Error> =
                            serde_json::from_str(&event_text);
                        if let Ok(decoded) = decoded {
                            self.data.borrow_mut().events.extend(decoded.logs);
                        }
                    }
                    WsMessage::Unknown(str_error) => {
                        self.error = true;
                        log::error!("Unknown message: {:?}", str_error);
                        self.error_str = str_error;
                    }
                    WsMessage::Binary(bin) => {
                        self.error = true;
                        self.error_str = format!("{:?}", bin);
                    }
                    _ => {
                        self.error = true;
                        self.error_str = "Received Ping-Pong".to_string();
                    }
                },
                WsEvent::Opened => {
                    self.connected = true;
                }
                WsEvent::Closed => {
                    self.connected = false;
                }
                WsEvent::Error(str_err) => {
                    self.connected = false;
                    self.error = true;
                    log::error!("Unknown message: {:?}", str_err);
                    self.error_str = str_err;
                }
            }
        }
        if self.connected {
            for one_window in self.windows.iter_mut() {
                let mut is_open: bool = self.data.borrow().open_windows.contains(one_window.name());
                one_window.show(ctx, &mut is_open);
                set_open(
                    &mut self.data.borrow_mut().open_windows,
                    one_window.name(),
                    is_open,
                );
            }
            egui::CentralPanel::default().show(ctx, |_ui| {});
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Not connected");
            });
        }
    }
}
