use crate::panels::{AboutPanel, LogicalChannels, MessageBox, PanelController, SocketManager};
use crate::set_open;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeSet};
use tramex_tools::types::internals::Interface;
use tramex_tools::{connector::Connector, types::websocket_types::WsConnection};
pub struct FrontEnd {
    pub connector: Rc<RefCell<Connector>>,
    pub open_windows: BTreeSet<String>,
    pub windows: Vec<Box<dyn PanelController>>,
    pub error_str: Option<String>,
}

impl FrontEnd {
    pub fn new(ws_sender: ewebsock::WsSender, ws_receiver: ewebsock::WsReceiver) -> Self {
        let ws = WsConnection {
            ws_receiver: Box::new(ws_receiver),
            ws_sender: Box::new(ws_sender),
            connecting: true,
            error_str: None,
        };
        let connector = Connector::new_ws(ws);
        let ref_connector = Rc::new(RefCell::new(connector));
        let mb = MessageBox::new(Rc::clone(&ref_connector));
        let sm = SocketManager::new(Rc::clone(&ref_connector));
        let lc = LogicalChannels::new(Rc::clone(&ref_connector));
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<AboutPanel>::default(),
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<SocketManager>::new(sm),
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

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.connector.borrow_mut().try_recv();
        if let Interface::Ws(interface_ws) = &self.connector.borrow().interface {
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
            } else if self.connector.borrow().available {
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    one_window.show(ctx, &mut is_open);
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
                egui::CentralPanel::default().show(ctx, |_ui| {});
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
