//! Message panel
use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::PanelView;
use eframe::egui;

#[cfg(feature = "ai")]
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
#[cfg(feature = "ai")]
use tramex_tools::ai::{AIExplainStatus, AIProvider, create_connector};
use tramex_tools::{data::Trace, errors::TramexError};
#[cfg(feature = "types_lte_3gpp")]
use types_lte_3gpp::{
    export::asn1_codecs::{PerCodecData, uper::UperCodec},
    uper::spec_rrc,
};
/// Message box
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

    #[cfg(feature = "ai")]
    /// AI API key (synced from settings)
    ai_api_key: String,

    #[cfg(feature = "ai")]
    /// AI provider (synced from settings)
    ai_provider: AIProvider,

    #[cfg(feature = "ai")]
    /// AI model ID (synced from settings)
    ai_model: String,

    #[cfg(feature = "ai")]
    /// Current AI explanation status
    ai_status: AIExplainStatus,

    #[cfg(feature = "ai")]
    /// In-flight AI request promise
    ai_promise: Option<poll_promise::Promise<Result<String, String>>>,

    #[cfg(feature = "ai")]
    /// Markdown rendering cache
    commonmark_cache: CommonMarkCache,
}

impl Default for MessageBox {
    fn default() -> Self {
        Self {
            current_trace: None,
            events_len: 0,
            current_index: 0,
            show_full: false,
            save_text: Vec::new(),
            #[cfg(feature = "ai")]
            ai_api_key: String::new(),
            #[cfg(feature = "ai")]
            ai_provider: AIProvider::default(),
            #[cfg(feature = "ai")]
            ai_model: AIProvider::default().default_model().to_string(),
            #[cfg(feature = "ai")]
            ai_status: AIExplainStatus::default(),
            #[cfg(feature = "ai")]
            ai_promise: None,
            #[cfg(feature = "ai")]
            commonmark_cache: CommonMarkCache::default(),
        }
    }
}

impl MessageBox {
    /// Create a new MessageBox
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    #[cfg(feature = "ai")]
    /// Fire an AI explain request for the current trace
    fn request_ai_explain(&mut self) {
        let trace = match &self.current_trace {
            Some(t) => t.clone(),
            None => return,
        };

        let connector = create_connector(&self.ai_provider, &self.ai_model);
        let request = match connector.build_request(&trace, &self.ai_api_key) {
            Ok(r) => r,
            Err(e) => {
                self.ai_status = AIExplainStatus::Error(e.get_msg());
                return;
            }
        };

        // Build ehttp request
        let mut ehttp_req = ehttp::Request::post(&request.url, request.body.into_bytes());
        for (key, value) in &request.headers {
            ehttp_req.headers.insert(key.clone(), value.clone());
        }

        let (sender, promise) = poll_promise::Promise::new();
        let provider = self.ai_provider.clone();
        let model_clone = self.ai_model.clone();

        ehttp::fetch(ehttp_req, move |response| {
            let result = match response {
                Ok(resp) => {
                    let body = resp.text().unwrap_or("").to_string();
                    let conn = create_connector(&provider, &model_clone);
                    match conn.parse_response(&body) {
                        Ok(explanation) => Ok(explanation),
                        Err(e) => Err(e.get_msg()),
                    }
                }
                Err(err) => Err(format!("HTTP error: {err}")),
            };
            sender.send(result);
        });

        self.ai_promise = Some(promise);
        self.ai_status = AIExplainStatus::Loading;
    }

    #[cfg(feature = "ai")]
    /// Poll the in-flight AI promise and update status
    fn poll_ai_promise(&mut self) {
        let done = if let Some(promise) = &self.ai_promise {
            promise.ready().is_some()
        } else {
            false
        };

        if done && let Some(promise) = self.ai_promise.take() {
            match promise.block_and_take() {
                Ok(explanation) => {
                    self.ai_status = AIExplainStatus::Done(explanation);
                }
                Err(err) => {
                    self.ai_status = AIExplainStatus::Error(err);
                }
            }
        }
    }

    #[cfg(feature = "ai")]
    /// Render the AI explain button and response section
    fn ui_ai_section(&mut self, ui: &mut egui::Ui) {
        // Poll promise first
        self.poll_ai_promise();

        if self.current_trace.is_none() {
            return;
        }

        ui.separator();

        let can_request = self
            .ai_api_key
            .is_empty()
            .then_some("No API key set (configure in Settings > AI)")
            .or_else(|| matches!(self.ai_status, AIExplainStatus::Loading).then_some("Request in progress…"));

        ui.horizontal(|ui| {
            let button = egui::Button::new("AI Explain");
            let enabled = can_request.is_none();
            let response = ui.add_enabled(enabled, button);
            let clicked = response.clicked();
            if let Some(tooltip) = can_request {
                response.on_disabled_hover_text(tooltip);
            }
            if clicked {
                self.request_ai_explain();
            }

            match &self.ai_status {
                AIExplainStatus::Loading => {
                    ui.spinner();
                    ui.label("Thinking…");
                }
                AIExplainStatus::Idle => {}
                AIExplainStatus::Done(_) => {
                    ui.colored_label(egui::Color32::GREEN, "OK");
                }
                AIExplainStatus::Error(_) => {
                    ui.colored_label(egui::Color32::RED, "Error");
                }
            }
        });

        // Display response or error
        match &self.ai_status {
            AIExplainStatus::Done(text) => {
                ui.separator();
                let text_clone = text.clone();
                egui::ScrollArea::vertical()
                    .id_salt("scroll_area_ai")
                    .max_height(300.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        CommonMarkViewer::new()
                            .max_image_width(Some(512))
                            .show(ui, &mut self.commonmark_cache, &text_clone);
                    });
            }
            AIExplainStatus::Error(err) => {
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
            }
            _ => {}
        }
    }
}

impl super::PanelView for MessageBox {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if let Some(one_trace) = &self.current_trace {
            display_log(ui, one_trace, &mut self.show_full, &self.save_text);
        }

        #[cfg(feature = "ai")]
        self.ui_ai_section(ui);
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

    // Show full message checkbox and copy button
    ui.horizontal(|ui| {
        ui.checkbox(show_full, "Show full message");

        if *show_full {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(vec_text) = &curr_trace.text {
                    let full_text = vec_text.join("\n");
                    if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                        ui.ctx().copy_text(full_text);
                    }
                }
            });
        }
    });

    if *show_full {
        ui.separator();
        match &curr_trace.text {
            Some(vec_text) => {
                let full_text = vec_text.join("\n");
                let mut text_copy = full_text.as_str();
                egui::ScrollArea::vertical()
                    .id_salt("scroll_area_raw")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut text_copy)
                                .desired_width(f32::INFINITY)
                                // .font(egui::TextStyle::Monospace)
                                .interactive(true),
                        );
                    });
            }
            None => {
                ui.label("No text available for this trame");
            }
        }
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
    fn name(&self) -> &'static str {
        "Messages"
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {
        // MessageBox doesn't need to process metadata changes
    }

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

        // Reset AI state when navigating to a different trace
        #[cfg(feature = "ai")]
        {
            self.ai_status = AIExplainStatus::Idle;
            self.ai_promise = None;
        }

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
        #[cfg(feature = "ai")]
        {
            self.ai_status = AIExplainStatus::Idle;
            self.ai_promise = None;
        }
    }

    #[cfg(feature = "ai")]
    fn set_ai_config(&mut self, key: &str, provider: &tramex_tools::ai::AIProvider, model: &str) {
        self.ai_api_key = key.to_string();
        self.ai_provider = provider.clone();
        self.ai_model = model.to_string();
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
