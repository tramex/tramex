//! This module contains some utility functions used in the application.
#[cfg(feature = "types_lte_3gpp")]
use asn1_codecs::{uper::UperCodec, PerCodecData};
use egui::{text::LayoutJob, Color32, TextFormat, Ui};
use std::collections::BTreeSet;
use tramex_tools::data::Trace;
#[cfg(feature = "types_lte_3gpp")]
use types_lte_3gpp::uper::spec_rrc;

/// Create an hyperlink open in a new tab
pub fn make_hyperlink(ui: &mut egui::Ui, label: &str, url: &str, new_tab: bool) {
    use egui::widgets::*; // to use ui();
    egui::Hyperlink::from_label_and_url(label, url)
        .open_in_new_tab(new_tab)
        .ui(ui);
}
/// change the BTreeSet according to the boolean value
pub fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

/// Create a label with a background color
pub fn color_label(job: &mut LayoutJob, ui: &Ui, label: &str, need_color: bool) {
    let default_color = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };
    let background_color = if need_color { Color32::DARK_BLUE } else { Color32::DARK_RED };
    job.append(
        label,
        0.0,
        TextFormat {
            color: default_color,
            background: background_color,
            ..Default::default()
        },
    );
}

/// Display a Trace type
pub fn display_log(ui: &mut Ui, curr_trace: &Trace, full: bool) {
    ui.label(format!("{:?}", &curr_trace.trace_type));
    ui.label(format!("{:?}", &curr_trace.hexa));
    if full {
        ui.separator();
        match &curr_trace.text {
            Some(vec_text) => {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for elem in vec_text {
                            ui.label(elem);
                        }
                    });
            }
            None => {
                ui.label("No text available for this trame");
            }
        }
        #[cfg(feature = "types_lte_3gpp")]
        {
            let text = hexe_decoding(curr_trace);
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for line in text.replace("{", "{\n").replace(",", ",\n").split("\n") {
                        ui.label(line);
                    }
                });
        }
    }
}

/// Decode the hexa value with types_lte_3gpp
#[cfg(feature = "types_lte_3gpp")]
pub fn hexe_decoding(curr_trace: &Trace) -> String {
    let mut codec_data = PerCodecData::from_slice_uper(&curr_trace.hexa);
    let sib1 = spec_rrc::BCCH_BCH_Message::uper_decode(&mut codec_data);
    if let Ok(res) = sib1 {
        return format!("{:?}", res);
    }
    match (curr_trace.trace_type.canal.as_str(), curr_trace.trace_type.canal_msg.as_str()) {
        ("BCCH-BCH", "Master Information Block") => {}
        ("BCCH", "SIB1") => {}
        _ => return format!("No value"),
    }
    return format!("No value");
}
