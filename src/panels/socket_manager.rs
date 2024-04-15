use crate::Data;
use eframe::egui;
use ewebsock::WsMessage;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SocketManager {
    data: Rc<RefCell<Data>>,
    msg_id: u64,
    layers: Layers,
}

impl SocketManager {
    pub fn new(ws_sender: Rc<RefCell<Data>>) -> Self {
        Self {
            data: ws_sender,
            msg_id: 1,
            layers: Layers::new(),
        }
    }
    pub fn get_more_logs(&mut self) {
        let msg = LogGet::new(self.msg_id, self.layers.clone());
        self.msg_id += 1;
        if let Ok(msg_stringed) = serde_json::to_string(&msg) {
            log::info!("{}", msg_stringed);
            self.data
                .borrow_mut()
                .ws_sender
                .send(WsMessage::Text(msg_stringed));
        }
    }
}

impl super::PanelController for SocketManager {
    fn window_title(&self) -> &'static str {
        "Socket Manager"
    }
    fn name(&self) -> &'static str {
        "Socket Manager"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Layers {
    #[serde(rename(serialize = "PHY"))]
    phy: String,
    #[serde(rename(serialize = "MAC"))]
    mac: String,
    #[serde(rename(serialize = "RLC"))]
    rlc: String,
    #[serde(rename(serialize = "PDCP"))]
    pdcp: String,
    #[serde(rename(serialize = "RRC"))]
    rrc: String,
    #[serde(rename(serialize = "NAS"))]
    nas: String,
    #[serde(rename(serialize = "S72"))]
    s72: String,
    #[serde(rename(serialize = "S1AP"))]
    s1ap: String,
    #[serde(rename(serialize = "NGAP"))]
    ngap: String,
    #[serde(rename(serialize = "GTPU"))]
    gtpu: String,
    #[serde(rename(serialize = "X2AP"))]
    x2ap: String,
    #[serde(rename(serialize = "XnAP"))]
    xnap: String,
    #[serde(rename(serialize = "M2AP"))]
    m2ap: String,
    #[serde(rename(serialize = "LPPa"))]
    lppa: String,
    #[serde(rename(serialize = "NRPPa"))]
    nrppa: String,
    #[serde(rename(serialize = "TRX"))]
    trx: String,
}
impl Layers {
    pub fn new() -> Self {
        Self {
            phy: "debug".to_owned(),
            mac: "warn".to_owned(),
            rlc: "warn".to_owned(),
            pdcp: "warn".to_owned(),
            rrc: "debug".to_owned(),
            nas: "debug".to_owned(),
            s72: "warn".to_owned(),
            s1ap: "warn".to_owned(),
            ngap: "warn".to_owned(),
            gtpu: "warn".to_owned(),
            x2ap: "warn".to_owned(),
            xnap: "warn".to_owned(),
            m2ap: "warn".to_owned(),
            lppa: "warn".to_owned(),
            nrppa: "warn".to_owned(),
            trx: "warn".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LogGet {
    timeout: u64,
    min: u64,
    max: u64,
    layers: Layers,
    message: String,
    headers: bool,
    message_id: u64,
}

impl LogGet {
    pub fn new(id: u64, layers: Layers) -> Self {
        Self {
            timeout: 1,
            min: 64,
            max: 2048,
            layers: layers,
            message: "log_get".to_owned(),
            headers: false,
            message_id: id,
        }
    }
}

impl super::PanelView for SocketManager {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Previous").clicked() {
                log::info!("Previous");
                if self.data.borrow().current_index > 0 {
                    self.data.borrow_mut().current_index -= 1;
                }
            }
            if ui.button("Next").clicked() {
                log::info!("Next");
                if self.data.borrow().events.len() > self.data.borrow().current_index + 1 {
                    self.data.borrow_mut().current_index += 1;
                } else {
                    self.get_more_logs();
                }
            }
            if ui.button("More").clicked() {
                log::info!("More");
                self.get_more_logs();
            }
        });
        ui.collapsing("Layers options", |ui| {
            checkbox(ui, &mut self.layers.phy, "PHY");
            checkbox(ui, &mut self.layers.mac, "MAC");
            checkbox(ui, &mut self.layers.rlc, "RLC");
            checkbox(ui, &mut self.layers.pdcp, "PDCP");
            checkbox(ui, &mut self.layers.rrc, "RRC");
            checkbox(ui, &mut self.layers.nas, "NAS");
            checkbox(ui, &mut self.layers.s72, "S72");
            checkbox(ui, &mut self.layers.s1ap, "S1AP");
            checkbox(ui, &mut self.layers.ngap, "NGAP");
            checkbox(ui, &mut self.layers.gtpu, "GTPU");
            checkbox(ui, &mut self.layers.x2ap, "X2AP");
            checkbox(ui, &mut self.layers.xnap, "XnAP");
            checkbox(ui, &mut self.layers.m2ap, "M2AP");
            checkbox(ui, &mut self.layers.lppa, "LPPa");
            checkbox(ui, &mut self.layers.nrppa, "NRPPa");
            checkbox(ui, &mut self.layers.trx, "TRX");
        });
    }
}

fn checkbox(ui: &mut egui::Ui, string: &mut String, text: &str) {
    let mut checked = string == "debug";
    if ui.checkbox(&mut checked, text).changed() {
        if checked {
            *string = "debug".to_owned();
        } else {
            *string = "warn".to_owned();
        };
    };
}
