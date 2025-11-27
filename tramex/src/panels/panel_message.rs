//! Message panel
use eframe::egui;
use crate::event_system::{EventSubscriber, EventContext};
use crate::panels::PanelView;

use tramex_tools::{
    data::Trace,
    errors::TramexError,
    interface::{layer::Layer},
};
#[cfg(feature = "types_lte_3gpp")]
use types_lte_3gpp::{
    export::asn1_codecs::{PerCodecData, uper::UperCodec},
    uper::spec_rrc,
};
/// Message box
#[derive(Default)]
pub struct MessageBox {
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


impl super::PanelView for MessageBox {
    fn ui(&mut self, ui: &mut egui::Ui) {
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
/// NOTE: This function is deprecated since hexa field was removed from Trace
#[cfg(feature = "types_lte_3gpp")]
#[allow(dead_code)]
pub fn hexe_decoding(_curr_trace: &Trace) -> String {
    // Hexa field no longer exists in Trace
    "Hexe decoding not available".to_string()
}

// EventSubscriber implementation for new event system
impl EventSubscriber for MessageBox {
    fn on_event_added(&mut self, _event: &Trace, _index: usize, _context: &EventContext) {
        // MessageBox doesn't need to process every event as it arrives
        // It only displays the currently focused event
        // So this can be a no-op
        self.events_len = _context.all_events.len();
    }
    
    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates to an event, update the displayed message
        self.current_index = index;
        self.current_trace = Some(event.clone());
        self.events_len = _context.all_events.len();
        
        // Hexe decoding removed since hexa field no longer exists
        #[cfg(feature = "types_lte_3gpp")]
        {
            self.save_text = vec![];
        }
    }
    
    fn on_events_cleared(&mut self) {
        log::debug!("MessageBox: Clearing all state");
        self.current_trace = None;
        self.events_len = 0;
        self.current_index = 0;
        self.save_text.clear();
    }
    
    fn name(&self) -> &'static str {
        "Messages"
    }
    
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Messages")
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        Ok(())
    }
}
