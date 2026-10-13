//! Navigation Panel - Controls for navigating through events

use eframe::egui;
use tramex_tools::data::Trace;

/// Snapshot of application state needed by NavigationPanel
pub struct NavState<'a> {
    /// Current event index
    pub current_index: usize,
    /// Total loaded event count
    pub event_count: usize,
    /// Current trace (if any)
    pub current_event: Option<&'a Trace>,
    /// Total event count in source (if known)
    pub total_count: Option<usize>,
    /// Whether all data has been read
    pub is_fully_loaded: bool,
    /// WebSocket info: (is_ws_available, is_auto_loading)
    pub ws_info: Option<(bool, bool)>,
}

/// Navigation Panel for event navigation
pub struct NavigationPanel {
    /// Flag to trigger next navigation
    pub should_go_next: bool,
    /// Flag to trigger previous navigation
    pub should_go_previous: bool,
    /// Flag to toggle WebSocket auto-loading
    pub should_toggle_auto_loading: bool,
    /// Flag to trigger goto navigation (0-based index)
    pub should_goto_event: Option<usize>,
    /// Whether the index label is in edit mode
    editing_index: bool,
    /// Whether we just entered edit mode (for auto-focus)
    just_entered_edit: bool,
    /// Text buffer used while editing the index
    index_edit_buf: String,
}

impl NavigationPanel {
    /// Create a new navigation panel
    pub fn new() -> Self {
        Self {
            should_go_next: false,
            should_go_previous: false,
            should_toggle_auto_loading: false,
            should_goto_event: None,
            editing_index: false,
            just_entered_edit: false,
            index_edit_buf: String::new(),
        }
    }

    /// Show the navigation toolbar as a single horizontal row
    pub fn show_toolbar(&mut self, ui: &mut egui::Ui, state: &NavState<'_>) {
        let colors = crate::theme::ThemeColors::get(ui);

        ui.horizontal(|ui| {
            // Previous button
            ui.add_enabled_ui(state.current_index > 0, |ui| {
                let button = egui::Button::new(egui::RichText::new("◀ Previous").strong()).min_size(egui::vec2(90.0, 24.0));

                if ui.add(button).clicked() {
                    self.should_go_previous = true;
                }
            });

            // Next/Start button
            let button_text = if state.event_count == 0 { "▶ Start" } else { "▶ Next" };

            // Only disable Next button if at end AND fully loaded
            let is_at_end = state.event_count > 0 && state.current_index >= state.event_count - 1 && state.is_fully_loaded;

            ui.add_enabled_ui(!is_at_end, |ui| {
                let button = egui::Button::new(egui::RichText::new(button_text).color(colors.on_accent).strong())
                    .fill(colors.accent_fill)
                    .corner_radius(6.0)
                    .min_size(egui::vec2(90.0, 24.0));

                if ui.add(button).clicked() {
                    self.should_go_next = true;
                }
            });

            // WebSocket Resume/Pause button
            if let Some((is_ws_available, is_auto_loading)) = state.ws_info
                && is_ws_available
            {
                let button = if is_auto_loading {
                    egui::Button::new(egui::RichText::new("⏸ Pause").strong())
                } else {
                    egui::Button::new(egui::RichText::new("▶ Resume").color(colors.on_accent).strong())
                        .fill(colors.accent_fill)
                }
                .corner_radius(6.0)
                .min_size(egui::vec2(90.0, 24.0));

                if ui.add(button).clicked() {
                    self.should_toggle_auto_loading = true;
                }
            }

            ui.separator();

            let current_display = if state.event_count == 0 { 0 } else { state.current_index + 1 };
            let max_event = state.total_count.unwrap_or(state.event_count);

            ui.label("Event");

            // Clickable index / inline editor
            if self.editing_index {
                let input = egui::TextEdit::singleline(&mut self.index_edit_buf)
                    .desired_width(50.0)
                    .font(egui::TextStyle::Monospace);
                let response = ui.add(input);

                // Auto-focus only on the first frame
                if self.just_entered_edit {
                    response.request_focus();
                    self.just_entered_edit = false;
                }

                let committed = response.lost_focus();
                if committed {
                    // Validate and trigger goto
                    if let Some(n) = self
                        .index_edit_buf
                        .parse::<usize>()
                        .ok()
                        .filter(|&n| n >= 1 && n <= max_event)
                    {
                        self.should_goto_event = Some(n - 1);
                    }
                    self.editing_index = false;
                }
            } else {
                // Show as clickable label
                let label = egui::RichText::new(format!("{}", current_display))
                    .color(colors.accent)
                    .strong()
                    .underline();
                let response = ui.add(egui::Label::new(label).sense(egui::Sense::click()));
                if response.on_hover_text("Click to jump to event").clicked() {
                    self.editing_index = true;
                    self.just_entered_edit = true;
                    self.index_edit_buf = format!("{}", current_display);
                }
            }

            if max_event > 0 {
                ui.label(egui::RichText::new(format!("/ {}", max_event)).color(colors.text_weak));
            }

            if let Some(trace) = state.current_event {
                ui.separator();

                ui.label("Time");
                ui.label(
                    egui::RichText::new(Self::format_timestamp(trace.timestamp))
                        .color(colors.text)
                        .monospace(),
                );
            }
        });
    }

    /// Convert milliseconds to HH:MM:SS.FFF format
    fn format_timestamp(ms: i64) -> String {
        let total_seconds = ms / 1000;
        let milliseconds = ms % 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
    }
}

impl Default for NavigationPanel {
    fn default() -> Self {
        Self::new()
    }
}
