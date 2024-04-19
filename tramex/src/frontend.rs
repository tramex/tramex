use crate::panels::{FileHandler, LogicalChannels, MessageBox, PanelController, TrameManager};
use crate::set_open;
use egui::Ui;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeSet};
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::Interface;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Choice {
    File,
    WebSocket,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FrontEnd {
    pub connector: Rc<RefCell<Connector>>,
    pub open_windows: BTreeSet<String>,
    #[serde(skip)]
    pub windows: Vec<Box<dyn PanelController>>,
    pub open_menu_connector: bool,
    #[serde(skip)]
    file_upload: Option<FileHandler>,
    pub radio_choice: Choice,
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            connector: Rc::new(RefCell::new(Connector::new())),
            open_windows: BTreeSet::new(),
            windows: Vec::new(),
            open_menu_connector: true,
            radio_choice: Choice::WebSocket,
            file_upload: None,
        }
    }
}

impl FrontEnd {
    pub fn new() -> Self {
        let connector = Connector::new();
        let ref_connector = Rc::new(RefCell::new(connector));
        let mb = MessageBox::new(Rc::clone(&ref_connector));
        let sm = TrameManager::new(Rc::clone(&ref_connector));
        let lc = LogicalChannels::new(Rc::clone(&ref_connector));
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<TrameManager>::new(sm),
        ];
        let mut open_windows = BTreeSet::new();
        for one_box in wins.iter() {
            open_windows.insert(one_box.name().to_owned());
        }
        Self {
            connector: ref_connector,
            open_windows,
            windows: wins,
            ..Default::default()
        }
    }

    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.connector.borrow().available {
            ui.menu_button("Windows", |ui| {
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    ui.checkbox(&mut is_open, one_window.name());
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
            });
        }
    }

    pub fn show_url(&mut self, ui: &mut Ui, new_ctx: egui::Context) -> Result<(), TramexError> {
        let connector = &mut self.connector.borrow_mut();

        match &connector.interface {
            Interface::Ws(_interface_ws) => {
                ui.label("URL:");
                ui.label(&connector.url);
                if ui.button("Close").clicked() {
                    // close connection
                    match connector.interface.close() {
                        Ok(_) => {
                            connector.clear_interface();
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
            }
            _ => {}
        }

        match &connector.interface {
            Interface::Ws(interface_ws) => {
                if interface_ws.connecting {
                    ui.label("Connecting...");
                    ui.spinner();
                }
            }
            _ => {
                if (ui.text_edit_singleline(&mut connector.url).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.button("Connect").clicked()
                {
                    let wakup_fn = move || new_ctx.request_repaint(); // wake up UI thread on new message
                    let local_url = connector.url.clone();
                    if let Err(err) = connector.connect(&local_url, wakup_fn) {
                        return Err(err);
                    }
                }
            }
        }
        Ok(())
    }

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
                            ui.add_enabled_ui(self.connector.borrow().interface.is_none(), |ui| {
                                ui.label("Choose ws or file");
                                ui.radio_value(&mut self.radio_choice, Choice::File, "File");
                                ui.radio_value(
                                    &mut self.radio_choice,
                                    Choice::WebSocket,
                                    "WebSocket",
                                );
                            });
                        });
                        if save != self.radio_choice && self.radio_choice == Choice::File {
                            self.file_upload = Some(FileHandler::new());
                        }
                        ui.horizontal(|ui| match &self.radio_choice {
                            Choice::WebSocket => {
                                if let Err(err) = self.show_url(ui, ctx.clone()) {
                                    error = Some(err);
                                }
                            }
                            Choice::File => {
                                if let Some(file_handle) = &mut self.file_upload {
                                    match file_handle.ui(ui) {
                                        Ok(bo) => {
                                            if bo {
                                                if self.connector.borrow().interface.is_none() {
                                                    match file_handle.get_result() {
                                                        Ok(curr_file) => {
                                                            self.connector
                                                                .borrow_mut()
                                                                .set_file(curr_file);
                                                            file_handle.clear();
                                                        }
                                                        Err(err) => {
                                                            error = Some(TramexError::new(err, tramex_tools::errors::ErrorCode::FileErrorReadingFile));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            error = Some(TramexError::new(
                                                "Error in file upload".to_owned(),
                                                tramex_tools::errors::ErrorCode::FileNoFileSelected,
                                            ));
                                        }
                                    }
                                    if file_handle.get_picket_path().is_some() {
                                        if ui.button("Close").on_hover_text("Close file").clicked()
                                        {
                                            self.file_upload = None;
                                            self.connector.borrow_mut().clear_interface();
                                        }
                                    }
                                }
                            }
                        });
                    });
                    ui.separator();
                });
        }
        if let Some(e) = error {
            return Err(e);
        }
        Ok(())
    }

    pub fn ui(&mut self, ctx: &egui::Context) -> Result<(), TramexError> {
        if let Err(err) = self.connector.borrow_mut().try_recv() {
            egui::CentralPanel::default().show(ctx, |ui| ui.horizontal(|ui| ui.vertical(|_ui| {})));
            return Err(err);
        }
        if self.connector.borrow().available {
            for one_window in self.windows.iter_mut() {
                let mut is_open: bool = self.open_windows.contains(one_window.name());
                one_window.show(ctx, &mut is_open);
                set_open(&mut self.open_windows, one_window.name(), is_open);
            }
            egui::CentralPanel::default().show(ctx, |_ui| {});
        } else if let Interface::Ws(_interface_ws) = &self.connector.borrow().interface {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("WebSocket not available");
            });
        } else if let Interface::File(_interface_file) = &self.connector.borrow().interface {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("File not available");
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Not connected");
            });
        }
        Ok(())
    }
}
