use eframe::egui;
use std::cell::RefCell;
use std::rc::Rc;
use tramex_tools::connector::Connector;
use tramex_tools::types::websocket_types::Layers;

pub struct TrameManager {
    data: Rc<RefCell<Connector>>,
    msg_id: u64,
    layers: Layers,
}

impl TrameManager {
    pub fn new(connector: Rc<RefCell<Connector>>) -> Self {
        Self {
            data: connector,
            msg_id: 1,
            layers: Layers::new(),
        }
    }
}

impl super::PanelController for TrameManager {
    fn window_title(&self) -> &'static str {
        "Trame Manager"
    }
    fn name(&self) -> &'static str {
        "Trame Manager"
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

impl super::PanelView for TrameManager {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut should_get_more_log = false;
        ui.horizontal(|ui| {
            if ui.button("Previous").clicked() {
                log::info!("Previous");
                if self.data.borrow().data.current_index > 0 {
                    self.data.borrow_mut().data.current_index -= 1;
                }
            }
            if ui.button("Next").clicked() {
                log::info!("Next");
                if self.data.borrow().data.events.len() > self.data.borrow().data.current_index + 1
                {
                    self.data.borrow_mut().data.current_index += 1;
                } else {
                    should_get_more_log = true;
                }
            }
            if ui.button("More").clicked() {
                log::info!("More");
                should_get_more_log = true;
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
        if should_get_more_log {
            self.data
                .borrow_mut()
                .get_more_data(self.msg_id, self.layers.clone());
            self.msg_id += 1;
        }
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
