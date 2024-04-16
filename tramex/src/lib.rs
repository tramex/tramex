mod app;
pub mod panels;
pub use app::ExampleApp;
mod frontend;

use ewebsock::WsSender;
use std::collections::BTreeSet;
mod utils;
pub use utils::*;

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
