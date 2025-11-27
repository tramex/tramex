//! Navigation Panel - Controls for navigating through events

use eframe::egui;
use tramex_tools::data::Data;
use crate::panels::PanelController;
use tramex_tools::errors::TramexError;

/// Navigation Panel for event navigation
pub struct NavigationPanel {
    /// Flag to trigger next navigation
    pub should_go_next: bool,
    /// Flag to trigger previous navigation
    pub should_go_previous: bool,
    /// Flag to toggle WebSocket auto-loading
    pub should_toggle_auto_loading: bool,
    /// Total event count (if known)
    pub total_count: Option<usize>,
    /// Whether all data has been read from file
    pub is_full_read: bool,
}

impl NavigationPanel {
    /// Create a new navigation panel
    pub fn new() -> Self {
        Self {
            should_go_next: false,
            should_go_previous: false,
            should_toggle_auto_loading: false,
            total_count: None,
            is_full_read: false,
        }
    }
    
    /// Show the navigation panel with optional WebSocket state info
    pub fn show_with_ws_info(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data: &mut Data,
        ws_info: Option<(bool, bool)>, // (is_websocket_available, is_auto_loading)
    ) -> Result<(), TramexError> {
        egui::Window::new(self.window_title())
            .open(open)
            .resizable(true)
            .default_width(350.0)
            .show(ctx, |ui| {
                // Everything in one horizontal row for maximum compactness
                ui.horizontal(|ui| {
                    // Statistics section
                    let current_display = if data.events.is_empty() {
                        0
                    } else {
                        data.current_index + 1
                    };
                    
                    ui.label(egui::RichText::new("Event:").size(14.0));
                    if let Some(total) = self.total_count {
                        ui.label(egui::RichText::new(format!("{} / {}", current_display, total))
                            .color(egui::Color32::from_rgb(100, 200, 255))
                            .strong()
                            .size(15.0));
                    } else {
                        ui.label(egui::RichText::new(format!("{}", current_display))
                            .color(egui::Color32::from_rgb(100, 200, 255))
                            .strong()
                            .size(15.0));
                    }
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("Loaded:").size(14.0));
                    ui.label(egui::RichText::new(format!("{}", data.events.len()))
                        .color(egui::Color32::from_rgb(255, 200, 100))
                        .size(15.0));
                    
                    if let Some(trace) = data.events.get(data.current_index) {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.label(egui::RichText::new("Time:").size(14.0));
                        ui.label(egui::RichText::new(Self::format_timestamp(trace.timestamp))
                            .color(egui::Color32::from_rgb(200, 150, 255))
                            .monospace()
                            .size(14.0));
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.label(egui::RichText::new("Layer:").size(14.0));
                        ui.label(egui::RichText::new(format!("{:?}", trace.layer))
                            .color(egui::Color32::from_rgb(255, 150, 150))
                            .strong()
                            .size(14.0));
                    }
                });
                
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
                
                // Navigation Controls
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        // Previous button
                        ui.add_enabled_ui(data.current_index > 0, |ui| {
                            let button = egui::Button::new(
                                egui::RichText::new("◀ Previous")
                                    .color(egui::Color32::WHITE)
                                    .strong()
                                    .size(15.0)
                            )
                            .fill(egui::Color32::from_rgb(150, 100, 50))
                            .rounding(8.0)
                            .min_size(egui::vec2(150.0, 40.0));
                            
                            if ui.add(button).clicked() {
                                self.should_go_previous = true;
                            }
                        });
                        
                        ui.add_space(10.0);
                        
                        // Next/Start button
                        let (button_text, button_color) = if data.events.is_empty() {
                            ("▶ Start", egui::Color32::from_rgb(50, 180, 50))
                        } else {
                            ("▶ Next", egui::Color32::from_rgb(50, 120, 220))
                        };
                        
                        // Only disable Next button if at end of file AND file is fully read
                        // If file is not fully read, we can load more batches
                        let is_at_end_of_file = !data.events.is_empty() 
                            && data.current_index >= data.events.len() - 1 
                            && self.is_full_read;
                        
                        ui.add_enabled_ui(!is_at_end_of_file, |ui| {
                            let button = egui::Button::new(
                                egui::RichText::new(button_text)
                                    .color(egui::Color32::WHITE)
                                    .strong()
                                    .size(15.0)
                            )
                            .fill(button_color)
                            .rounding(8.0)
                            .min_size(egui::vec2(150.0, 40.0));
                            
                            if ui.add(button).clicked() {
                                self.should_go_next = true;
                            }
                        });
                        
                        // WebSocket Resume/Pause button (aligned with Previous/Next)
                        if let Some((is_ws_available, is_auto_loading)) = ws_info {
                            if is_ws_available {
                                ui.add_space(10.0);
                                
                                let (button_text, button_color) = if is_auto_loading {
                                    ("⏸ Pause", egui::Color32::from_rgb(120, 120, 120))
                                } else {
                                    ("▶ Resume", egui::Color32::from_rgb(50, 180, 50))
                                };
                                
                                let button = egui::Button::new(
                                    egui::RichText::new(button_text)
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                        .size(15.0)
                                )
                                .fill(button_color)
                                .rounding(8.0)
                                .min_size(egui::vec2(150.0, 40.0));  // Same width as Previous/Next
                                
                                if ui.add(button).clicked() {
                                    self.should_toggle_auto_loading = true;
                                }
                            }
                        }
                    });
                    
                    // Status indicator (outside horizontal layout, centered below buttons)
                    if let Some((is_ws_available, is_auto_loading)) = ws_info {
                        if is_ws_available {
                            ui.add_space(5.0);
                            if is_auto_loading {
                                ui.label(egui::RichText::new("🔄 Auto-loading enabled")
                                    .color(egui::Color32::from_rgb(100, 200, 100)));
                            } else {
                                ui.label(egui::RichText::new("⏸ Auto-loading paused")
                                    .color(egui::Color32::from_rgb(200, 150, 100)));
                            }
                        }
                    }
                });
            });
        
        Ok(())
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
    
    /// Show navigation controls (kept for compatibility but not used)
    #[allow(dead_code)]
    pub fn show_controls(
        &mut self,
        ui: &mut egui::Ui,
        data: &Data,
        _on_start: &mut bool,
        on_next: &mut bool,
        on_previous: &mut bool,
        is_full_read: bool,
        total_count: Option<usize>,
    ) {
        ui.heading("📊 Event Navigation");
        ui.separator();
        
        // Statistics section
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Statistics").strong());
                ui.separator();
                
                // Current event index
                let current_display = if data.events.is_empty() {
                    0
                } else {
                    data.current_index + 1
                };
                
                ui.horizontal(|ui| {
                    ui.label("Current Event:");
                    ui.label(egui::RichText::new(format!("{}", current_display))
                        .color(egui::Color32::from_rgb(100, 200, 255))
                        .strong());
                });
                
                // Total count
                if let Some(total) = total_count {
                    ui.horizontal(|ui| {
                        ui.label("Total Events:");
                        ui.label(egui::RichText::new(format!("{}", total))
                            .color(egui::Color32::from_rgb(100, 255, 100))
                            .strong());
                    });
                    
                    // Progress bar
                    if total > 0 {
                        let progress = current_display as f32 / total as f32;
                        ui.add(egui::ProgressBar::new(progress)
                            .text(format!("{:.1}%", progress * 100.0)));
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Total Events:");
                        ui.label(egui::RichText::new("Unknown")
                            .color(egui::Color32::GRAY)
                            .italics());
                    });
                }
                
                // Received events (loaded in memory)
                ui.horizontal(|ui| {
                    ui.label("Loaded Events:");
                    ui.label(egui::RichText::new(format!("{}", data.events.len()))
                        .color(egui::Color32::from_rgb(255, 200, 100)));
                });
                
                // Current timestamp
                if let Some(trace) = data.events.get(data.current_index) {
                    ui.horizontal(|ui| {
                        ui.label("Timestamp:");
                        ui.label(egui::RichText::new(format!("{}", trace.timestamp))
                            .color(egui::Color32::from_rgb(200, 150, 255))
                            .monospace());
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Layer:");
                        ui.label(egui::RichText::new(format!("{:?}", trace.layer))
                            .color(egui::Color32::from_rgb(255, 150, 150))
                            .strong());
                    });
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Navigation controls
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Controls").strong());
                ui.separator();
                
                ui.horizontal(|ui| {
                    // Start/Next button
                    let is_enabled = if !data.events.is_empty() && data.events.len() - 1 == data.current_index {
                        !is_full_read
                    } else {
                        true
                    };
                    
                    let button_text = if data.events.is_empty() {
                        "▶ Start"
                    } else {
                        "Next ▶"
                    };
                    
                    ui.add_enabled_ui(is_enabled, |ui| {
                        let button = egui::Button::new(
                            egui::RichText::new(button_text)
                                .color(egui::Color32::WHITE)
                                .strong()
                        )
                        .fill(egui::Color32::from_rgb(50, 150, 50))
                        .min_size(egui::vec2(120.0, 35.0));
                        
                        if ui.add(button).clicked() {
                            *on_next = true;
                        }
                    });
                    
                    // Previous button
                    ui.add_enabled_ui(data.current_index > 0, |ui| {
                        let button = egui::Button::new(
                            egui::RichText::new("◀ Previous")
                                .color(egui::Color32::WHITE)
                                .strong()
                        )
                        .fill(egui::Color32::from_rgb(150, 100, 50))
                        .min_size(egui::vec2(120.0, 35.0));
                        
                        if ui.add(button).clicked() {
                            *on_previous = true;
                        }
                    });
                });
                
                // Status indicator
                ui.add_space(5.0);
                if is_full_read {
                    ui.label(egui::RichText::new("✓ All events loaded")
                        .color(egui::Color32::from_rgb(100, 200, 100)));
                } else {
                    ui.label(egui::RichText::new("⟳ Loading more events...")
                        .color(egui::Color32::from_rgb(200, 200, 100)));
                }
            });
        });
    }
}

impl Default for NavigationPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelController for NavigationPanel {
    fn name(&self) -> &'static str {
        "Navigation"
    }
    
    fn window_title(&self) -> &'static str {
        "📊 Navigation Panel"
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data: &mut Data,
    ) -> Result<(), TramexError> {
        // Delegate to show_with_ws_info with no WebSocket info
        self.show_with_ws_info(ctx, open, data, None)
    }

    fn clear(&mut self) {
        // Nothing to clear
    }
}
