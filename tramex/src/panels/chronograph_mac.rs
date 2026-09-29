//! Chronograph MAC-RLC-PDCP Panel
//!
//! Displays a visual timeline of MAC, RLC, and PDCP message exchanges between UE and BST

use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::NAVIGATE_REQUEST_ID;
use crate::theme::{ArrowColors, ThemeColors};
use egui::{self, CursorIcon, PopupAnchor, Pos2, Rect, Stroke, Vec2};
use tramex_tools::{
    data::Trace,
    errors::TramexError,
    interface::{layer::Layer, types::Direction},
};

/// Hardware element in the chronograph (only UE and BST for MAC/RLC/PDCP)
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum Hardware {
    /// User Equipment
    UE,
    /// Base Station
    BST,
}

/// Arrow representation for a message exchange
#[derive(Debug, Clone)]
struct MessageArrow {
    /// Index in the trace list
    trace_index: usize,
    /// Source hardware
    from: Hardware,
    /// Destination hardware
    to: Hardware,
    /// Layer type
    layer: Layer,
    /// Full message content (direction + raw data)
    message: String,
}

impl MessageArrow {
    /// Determine arrow direction based on layer and direction
    fn from_trace(trace: &Trace, index: usize) -> Option<Self> {
        // Only handle MAC, RLC, and PDCP layers
        let layer = match &trace.layer {
            Layer::MAC | Layer::RLC | Layer::PDCP => trace.layer.clone(),
            _ => return None,
        };

        // For MAC/RLC/PDCP, direction is extracted from the raw text
        // Format: "HH:MM:SS.mmm [LAYER] DIR ..."
        let direction = Self::extract_direction_from_text(trace)?;

        let (from, to) = match direction {
            Direction::UL => (Hardware::UE, Hardware::BST),
            Direction::DL => (Hardware::BST, Hardware::UE),
            _ => return None,
        };

        // Extract the full message from raw text (everything after direction)
        let message = Self::extract_message_from_text(trace, &direction);

        Some(MessageArrow {
            trace_index: index,
            from,
            to,
            layer,
            message,
        })
    }

    /// Extract direction from trace text
    fn extract_direction_from_text(trace: &Trace) -> Option<Direction> {
        // First check if additional_infos has direction
        if let Some(dir) = trace.additional_infos.get_direction() {
            return Some(dir);
        }

        // Otherwise, parse from raw text
        let text = trace.text.as_ref()?;
        let first_line = text.first()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        // Format: "HH:MM:SS.mmm [LAYER] DIR ..."
        if parts.len() >= 3 {
            match parts[2] {
                "UL" => Some(Direction::UL),
                "DL" => Some(Direction::DL),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Extract the message content from trace text
    fn extract_message_from_text(trace: &Trace, direction: &Direction) -> String {
        let dir_str = match direction {
            Direction::UL => "UL",
            Direction::DL => "DL",
            _ => "",
        };

        if let Some(text) = &trace.text
            && let Some(first_line) = text.first()
        {
            // Format: "HH:MM:SS.mmm [LAYER] DIR rest_of_message"
            // We want to extract everything after [LAYER] (including direction)
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 4 {
                // Skip timestamp and layer, take direction + rest
                let msg_parts: Vec<&str> = parts[2..].to_vec();
                return msg_parts.join(" ");
            }
        }

        // Fallback: just show direction and layer
        format!("{} {:?}", dir_str, trace.layer)
    }
}

/// Chronograph MAC-RLC-PDCP Panel
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChronographMac {
    /// Current trace index
    #[serde(skip)]
    current_index: usize,

    /// Cached arrows (to avoid regenerating when navigating back)
    #[serde(skip)]
    arrows: Vec<MessageArrow>,

    /// Scroll offset
    scroll_offset: f32,

    /// Flag to trigger scroll on next frame
    should_scroll: bool,

    /// Parent trace index of the current focused trace (if any)
    #[serde(skip)]
    related_parent: Option<Vec<usize>>,

    /// Child trace index of the current focused trace (if any)
    #[serde(skip)]
    related_child: Option<Vec<usize>>,

    /// Hovered arrow index (for click-to-navigate)
    #[serde(skip)]
    hovered_arrow: Option<usize>,
}

impl Default for ChronographMac {
    fn default() -> Self {
        Self::new()
    }
}

impl ChronographMac {
    /// Create a new ChronographMac panel
    pub fn new() -> Self {
        Self {
            current_index: 0,
            arrows: Vec::new(),
            scroll_offset: 0.0,
            should_scroll: false,
            related_parent: None,
            related_child: None,
            hovered_arrow: None,
        }
    }

    /// Get X position for a hardware element
    fn get_hardware_x(hardware: Hardware, rect: &Rect) -> f32 {
        let width = rect.width();
        let padding = width * 0.15; // 15% padding on each side
        match hardware {
            Hardware::UE => rect.left() + padding,
            Hardware::BST => rect.right() - padding,
        }
    }

    /// Draw the chronograph
    fn draw_chronograph(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        // Use a scroll area for the timeline
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);

        // Conditionally scroll to current arrow if flag is set
        let scroll_area = if self.should_scroll {
            if let Some(current_arrow_idx) = self.arrows.iter().position(|a| a.trace_index == self.current_index) {
                let arrow_height = 40.0;
                let header_height = 50.0;
                let arrow_y_position = (current_arrow_idx as f32 * arrow_height) + header_height + 10.0;

                // Center the arrow in the viewport by subtracting half the viewport height
                let viewport_height = available_size.y;
                let centered_offset = (arrow_y_position - viewport_height / 2.0).max(0.0);

                log::debug!(
                    "ChronographMac: Scrolling to centered offset {} (arrow at index {})",
                    centered_offset,
                    self.current_index
                );
                scroll_area.vertical_scroll_offset(centered_offset)
            } else {
                scroll_area
            }
        } else {
            scroll_area
        };

        // Reset hovered arrow before drawing
        self.hovered_arrow = None;

        scroll_area.show(ui, |ui| {
            // Reserve space for all arrows + padding
            let arrow_height = 40.0;
            let total_height = self.arrows.len() as f32 * arrow_height + 100.0;
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(available_size.x - 20.0, total_height), egui::Sense::click());

            let painter = ui.painter();

            // Add padding at the top for headers
            let header_height = 50.0;
            let arrow_start_y = rect.top() + header_height;

            // Draw vertical lines for hardware (starting after header)
            let theme = ThemeColors::get(ui);
            let line_stroke = Stroke::new(2.0, theme.text_weak);

            let ue_x = Self::get_hardware_x(Hardware::UE, &rect);
            let bst_x = Self::get_hardware_x(Hardware::BST, &rect);

            painter.line_segment([Pos2::new(ue_x, arrow_start_y), Pos2::new(ue_x, rect.bottom())], line_stroke);
            painter.line_segment(
                [Pos2::new(bst_x, arrow_start_y), Pos2::new(bst_x, rect.bottom())],
                line_stroke,
            );

            // Draw labels at the top - use theme-aware text color
            let text_color = theme.text;
            painter.text(
                Pos2::new(ue_x, rect.top() + 20.0),
                egui::Align2::CENTER_CENTER,
                "UE",
                egui::FontId::proportional(16.0),
                text_color,
            );
            painter.text(
                Pos2::new(bst_x, rect.top() + 20.0),
                egui::Align2::CENTER_CENTER,
                "BST",
                egui::FontId::proportional(16.0),
                text_color,
            );

            // Get mouse position for hover detection
            let mouse_pos = ui.ctx().pointer_hover_pos();

            // Draw arrows
            let start_y = arrow_start_y + 10.0;
            for (i, arrow) in self.arrows.iter().enumerate() {
                let y = start_y + (i as f32 * arrow_height);
                let is_current: bool = arrow.trace_index == self.current_index;
                let is_related_parent = self.related_parent.as_ref().is_some_and(|v| v.contains(&arrow.trace_index));
                let is_related_child = self.related_child.as_ref().is_some_and(|v| v.contains(&arrow.trace_index));

                let from_x = Self::get_hardware_x(arrow.from, &rect);
                let to_x = Self::get_hardware_x(arrow.to, &rect);

                // Check if mouse is hovering over this arrow
                let arrow_rect = Rect::from_min_max(
                    Pos2::new(from_x.min(to_x) - 10.0, y - 15.0),
                    Pos2::new(from_x.max(to_x) + 10.0, y + 15.0),
                );
                let is_hovered = mouse_pos.is_some_and(|pos| arrow_rect.contains(pos));

                if is_hovered {
                    self.hovered_arrow = Some(i);
                }

                // Determine arrow color and thickness
                // Current: bright blue, Related (parent/child): lighter blue, Hovered: highlight, Others: theme-aware
                let (arrow_color, arrow_width) = if is_current {
                    (ArrowColors::CURRENT, 3.0) // Blue and thicker for current
                } else if is_hovered {
                    (ArrowColors::CURRENT, 2.5) // Highlight on hover
                } else if is_related_parent || is_related_child {
                    (ArrowColors::RELATED, 2.5) // Lighter blue for related traces
                } else {
                    (theme.text, 1.5) // Theme-aware for others
                };

                // Draw arrow line
                painter.line_segment(
                    [Pos2::new(from_x, y), Pos2::new(to_x, y)],
                    Stroke::new(arrow_width, arrow_color),
                );

                // Draw arrowhead
                let arrow_size = 8.0;
                let direction = if to_x > from_x { 1.0 } else { -1.0 };
                let tip = Pos2::new(to_x, y);
                let base1 = Pos2::new(to_x - direction * arrow_size, y - arrow_size / 2.0);
                let base2 = Pos2::new(to_x - direction * arrow_size, y + arrow_size / 2.0);

                painter.add(egui::Shape::convex_polygon(
                    vec![tip, base1, base2],
                    arrow_color,
                    Stroke::NONE,
                ));

                // Draw message text on top of arrow
                // Format: "LAYER - message"
                let text = format!("{:?} - {}", arrow.layer, arrow.message);
                let text_pos = Pos2::new((from_x + to_x) / 2.0, y - 10.0);

                // Truncate text if too long
                let max_chars = 60;
                let display_text = if text.len() > max_chars {
                    format!("{}...", &text[..max_chars])
                } else {
                    text
                };

                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    display_text,
                    egui::FontId::proportional(10.0),
                    arrow_color,
                );
            }

            // Handle hover cursor and click-to-navigate
            if let Some(hovered_idx) = self.hovered_arrow {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);

                // Show tooltip with full message
                let arrow = &self.arrows[hovered_idx];
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    egui::Id::new("chronograph_mac_tooltip"),
                    PopupAnchor::Pointer,
                )
                .at_pointer()
                .show(|ui| {
                    ui.label(egui::RichText::new(format!("{:?}", arrow.layer)).strong());
                    ui.label(&arrow.message);
                    ui.label(egui::RichText::new("Click to navigate").small().weak());
                });

                // Handle click
                if response.clicked() {
                    let trace_index = self.arrows[hovered_idx].trace_index;
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(egui::Id::new(NAVIGATE_REQUEST_ID), trace_index));
                    log::debug!("ChronographMac: Click-to-navigate to trace {}", trace_index);
                }
            }
        });

        // Reset scroll flag after drawing
        self.should_scroll = false;
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for ChronographMac {
    fn name(&self) -> &'static str {
        "Chronograph MAC-RLC-PDCP"
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {}

    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When a new event is added, create an arrow for it
        if let Some(arrow) = MessageArrow::from_trace(event, index) {
            // Check if arrow already exists (avoid duplicates when changing filters)
            if self.arrows.iter().any(|a| a.trace_index == index) {
                log::trace!("ChronographMac: Arrow for event {} already exists, skipping", index);
                return;
            }

            // Find the correct position to insert based on trace_index
            // This ensures arrows are always in chronological order
            let insert_pos = self
                .arrows
                .iter()
                .position(|a| a.trace_index > index)
                .unwrap_or(self.arrows.len());

            log::trace!(
                "ChronographMac: Inserting arrow at position {} for event {} (layer: {:?})",
                insert_pos,
                index,
                event.layer
            );

            self.arrows.insert(insert_pos, arrow);

            // Limit arrow history (remove oldest)
            if self.arrows.len() > 500 {
                self.arrows.remove(0);
            }
        }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates to an event, update current index and scroll
        self.current_index = index;
        self.should_scroll = true;

        // Update related parent/child indices for highlighting
        self.related_parent = event.relation.get_parent_indices().cloned();
        self.related_child = event.relation.get_child_indices().cloned();

        log::trace!(
            "ChronographMac: Focused on event {}, parent={:?}, child={:?}",
            index,
            self.related_parent,
            self.related_child
        );
    }

    fn on_events_cleared(&mut self) {
        log::debug!("ChronographMac: Clearing all arrows");
        self.arrows.clear();
        self.current_index = 0;
        self.scroll_offset = 0.0;
        self.should_scroll = false;
        self.related_parent = None;
        self.related_child = None;
        self.hovered_arrow = None;
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Chronograph MAC-RLC-PDCP")
            .resizable(true)
            .default_width(640.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                self.draw_chronograph(ui);
            });
        Ok(())
    }
}

// PanelView implementation for rendering UI
impl super::PanelView for ChronographMac {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.draw_chronograph(ui);
    }
}
