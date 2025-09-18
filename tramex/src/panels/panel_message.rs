//! Message panel
use crate::display_log;
use eframe::egui;
use tramex_tools::{
    data::{Data, Trace},
    errors::TramexError,
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
        ui.heading("Received events:");
        ui.checkbox(&mut self.show_full, "Show full message");
        ui.horizontal(|ui| {
            ui.label(format!("Received events: {}", self.events_len));
        });

        if let Some(one_trace) = &self.current_trace {
            ui.label(format!("Current msg index: {}", self.current_index + 1));
            display_log(ui, one_trace, self.show_full, &self.save_text);
        }
    }
}
