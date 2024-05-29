use std::collections::BTreeSet;

use egui::{text::LayoutJob, Color32, TextFormat, Ui};
use tramex_tools::data::Trace;

pub fn make_hyperlink(ui: &mut egui::Ui, label: &str, url: &str, new_tab: bool) {
    use egui::widgets::*; // to use ui();
    egui::Hyperlink::from_label_and_url(label, url)
        .open_in_new_tab(new_tab)
        .ui(ui);
}

pub fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

pub fn color_label(job: &mut LayoutJob, ui: &Ui, label: &str, need_color: bool) {
    let default_color = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };
    let background = if need_color { Color32::DARK_BLUE } else { Color32::DARK_RED };
    job.append(
        label,
        0.0,
        TextFormat {
            color: default_color,
            background: background,
            ..Default::default()
        },
    );
}

pub fn display_log(ui: &mut Ui, log: &Trace) {
    let job = LayoutJob::default();
    ui.label(format!("{:?}", &log.trace_type));
    ui.label(format!("{:?}", &log.hexa));
    ui.label(job);
}
