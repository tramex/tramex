mod app;
pub mod panels;
pub use app::ExampleApp;

use ewebsock::WsSender;
use std::collections::BTreeSet;

pub struct Data {
    pub ws_sender: WsSender,
    pub events: Vec<OneLog>,
    pub open_windows: BTreeSet<String>,
    pub current_index: usize,
}

#[derive(serde::Deserialize, Debug)]
pub struct OneLog {
    pub data: Vec<String>,   // Each item is a string representing a line of log.
    pub timestamp: u64,      // Milliseconds since January 1st 1970.
    pub layer: String,       // log layer
    pub level: u64,          // Log level: error, warn, info or debug.
    pub dir: Option<String>, //  Log direction: UL, DL, FROM or TO.
    pub cell: Option<u64>,   // cell id
    pub channel: Option<String>, // channels
    pub src: String,
    pub idx: u64,
}

// deserialize the message
#[derive(serde::Deserialize, Debug)]
pub struct WebSocketLog {
    pub logs: Vec<OneLog>,
}

use egui::text::LayoutJob;
use egui::{Color32, TextFormat, Ui};

fn color_label(job: &mut LayoutJob, ui: &Ui, label: &str, need_color: bool) {
    let default_color = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };
    let background = if need_color {
        Color32::DARK_BLUE
    } else {
        Color32::DARK_RED
    };
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

pub fn display_log(ui: &mut Ui, log: &OneLog) {
    let job = LayoutJob::default();
    let data_type = match log.data.len() {
        0 => None,
        _ => Some(&log.data[0]),
    };
    if let Some(data_type) = data_type {
        ui.label(data_type);
    }

    let data: Vec<&str> = log
        .data
        .iter()
        .filter(|one_string| {
            if let Some(first_char) = one_string.chars().next() {
                return first_char.is_numeric();
            }
            return false;
        })
        .map(|one_string| {
            if one_string.len() > 57 {
                let str = &one_string[6..57];
                return str;
            }
            return "";
        })
        .collect();
    for one_data in data {
        ui.label(one_data);
    }
    ui.label(job);
}
