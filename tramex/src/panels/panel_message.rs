//! Message panel
use eframe::egui;

use tramex_tools::{
    data::{Data, Trace},
    errors::TramexError,
    interface::{layer::Layer, parse_config::Technology},
};
#[cfg(feature = "types_lte_3gpp")]
use types_lte_3gpp::{
    export::asn1_codecs::{PerCodecData, uper::UperCodec},
    uper::spec_rrc,
};
/// Message box
#[derive(Default)]
pub struct MessageBox {
    /// technology (LTE or NR)
    technology: Technology,

    /// current trace
    current_trace: Option<Trace>,

    /// events length
    events_len: usize,

    /// current index
    current_index: usize,

    /// show full message
    show_full: bool,

    /// save text
    save_text: Vec<String>,
}

impl MessageBox {
    /// Create a new MessageBox
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
}

impl super::PanelController for MessageBox {
    fn name(&self) -> &'static str {
        "Messages"
    }

    fn window_title(&self) -> &'static str {
        "Current Message"
    }

    fn clear(&mut self) {
        self.current_trace = None;
        self.events_len = 0;
        self.current_index = 0;
        self.save_text = Vec::new();
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError> {
        // Update technology from data metadata
        self.technology = data.metadata.technology;
        if data.is_different_index(self.current_index) {
            if let Some(trace) = data.get_current_trace() {
                self.current_trace = Some(trace.clone());
                #[cfg(feature = "types_lte_3gpp")]
                {
                    let mut count = 0;
                    use crate::hexe_decoding;
                    self.save_text = hexe_decoding(trace)
                        .replace('{', "{\n")
                        .replace(',', ",\n")
                        .split('\n')
                        .map(|x| {
                            if x.contains('{') {
                                count += 1;
                            } else if x.contains('}') {
                                count -= 1;
                            }
                            format!("{} {}", " ".repeat(count * 4), x)
                        })
                        .collect();
                }
            }
            self.current_index = data.current_index;
        }
        self.events_len = data.events.len();
        self.current_index = data.current_index;
        egui::Window::new(self.window_title())
            .default_width(320.0)
            .default_height(480.0)
            .resizable(true)
            .open(open)
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui);
            });
        Ok(())
    }
}

impl super::PanelView for MessageBox {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("Technology : {}", self.technology));
        ui.separator();

        if let Some(one_trace) = &self.current_trace {
            display_log(ui, one_trace, &mut self.show_full, &self.save_text);
        }
    }
}

/// Display a Trace type
fn display_log(ui: &mut egui::Ui, curr_trace: &Trace, show_full: &mut bool, _text: &[String]) {
    // Display layer type in big heading
    ui.heading(format!("Layer : {:?}", &curr_trace.layer));
    ui.spacing();

    // Display additional infos
    ui.label(format!("{:?}", &curr_trace.additional_infos));
    ui.separator();

    // Show full message checkbox
    ui.checkbox(show_full, "Show full message");

    if *show_full {
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("scroll_area_raw")
            .max_height(250.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                // Display hex in full message zone
                if curr_trace.layer == Layer::RRC {
                    ui.label(format!("Hex: {:?}", &curr_trace.hexa));
                }
                
                match &curr_trace.text {
                    Some(vec_text) => {
                        for elem in vec_text {
                            ui.label(elem);
                        }
                    }
                    None => {
                        ui.label("No text available for this trame");
                    }
                }
            });
        #[cfg(feature = "types_lte_3gpp")]
        {
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("scroll_area_types")
                .max_height(250.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for line in _text {
                        ui.label(line);
                    }
                });
        }
    }
}

/// Decode the hexa value with types_lte_3gpp
#[cfg(feature = "types_lte_3gpp")]
pub fn hexe_decoding(curr_trace: &Trace) -> String {
    use tramex_tools::interface::layer::Layer;

    let mut codec_data = PerCodecData::from_slice_uper(&curr_trace.hexa);
    match curr_trace.layer {
        Layer::RRC => {
            // we should check the type of the message before decoding (TODO)
            let sib1 = spec_rrc::BCCH_BCH_Message::uper_decode(&mut codec_data);
            if let Ok(res) = sib1 {
                return format!("{:?}", res);
            }
            "No value".to_string()
        }
        _ => "Not implemented".to_string(),
    }
}
