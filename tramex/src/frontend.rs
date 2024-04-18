use crate::panels::{AboutPanel, LogicalChannels, MessageBox, PanelController, TrameManager};
use crate::set_open;
use egui::Ui;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeSet};
use tramex_tools::connector::Connector;
use tramex_tools::types::internals::Interface;

pub struct FrontEnd {
    pub connector: Rc<RefCell<Connector>>,
    pub open_windows: BTreeSet<String>,
    pub windows: Vec<Box<dyn PanelController>>,
    pub error_str: Option<String>,
}

impl FrontEnd {
    pub fn new() -> Self {
        let connector = Connector::new();
        let ref_connector = Rc::new(RefCell::new(connector));
        let mb = MessageBox::new(Rc::clone(&ref_connector));
        let sm = TrameManager::new(Rc::clone(&ref_connector));
        let lc = LogicalChannels::new(Rc::clone(&ref_connector));
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<AboutPanel>::default(),
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
            error_str: None,
        }
    }
    pub fn connect(&mut self, url: &str, wakup_fn: impl Fn() + Send + Sync + 'static) {
        if let Err(err) = self.connector.borrow_mut().connect(url, wakup_fn) {
            self.error_str = Some(err);
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

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.connector.borrow_mut().try_recv();
        if self.connector.borrow().available {
            for one_window in self.windows.iter_mut() {
                let mut is_open: bool = self.open_windows.contains(one_window.name());
                one_window.show(ctx, &mut is_open);
                set_open(&mut self.open_windows, one_window.name(), is_open);
            }
            egui::CentralPanel::default().show(ctx, |_ui| {});
        } else if let Interface::Ws(interface_ws) = &self.connector.borrow().interface {
            if let Some(err) = &interface_ws.error_str {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(err);
                });
            } else if interface_ws.connecting {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Connecting...");
                        ui.spinner();
                    });
                });
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Not connected");
                });
            }
        } else if let Interface::File(_interface_file) = &self.connector.borrow().interface {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("File lol");
            });
        }
    }
}
