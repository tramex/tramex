//! Chronograph Panel
//! 
//! Displays a visual timeline of message exchanges between UE, BST, and CN

use crate::event_system::{EventSubscriber, EventContext};
use egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use tramex_tools::{
    data::{Data, Trace},
    errors::TramexError,
    interface::{layer::Layer, types::Direction},
};

/// Hardware element in the chronograph
#[derive(Debug, Clone, Copy, PartialEq)]
enum Hardware {
    UE,   // User Equipment
    BST,  // Base Station
    CN,   // Core Network
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
    /// Message description
    message: String,
}

impl MessageArrow {
    /// Determine arrow direction based on layer and direction
    fn from_trace(trace: &Trace, index: usize) -> Option<Self> {
        let direction = trace.additional_infos.get_direction()?;
        let message = trace.additional_infos.get_message_name()?;
        
        let (from, to) = match trace.layer {
            Layer::RRC => {
                // RRC: UE <-> BST
                match direction {
                    Direction::UL => (Hardware::UE, Hardware::BST),
                    Direction::DL => (Hardware::BST, Hardware::UE),
                    _ => return None,
                }
            }
            Layer::NAS => {
                // NAS: UE <-> CN (direct line, conceptually through BST)
                match direction {
                    Direction::UL => (Hardware::UE, Hardware::CN),
                    Direction::DL => (Hardware::CN, Hardware::UE),
                    _ => return None,
                }
            }
            Layer::NGAP => {
                // NGAP: BST <-> CN
                // Check direction NGAP: TO = BST -> CN, FROM = CN -> BST
                match direction {
                    Direction::TO => (Hardware::BST, Hardware::CN),
                    Direction::FROM => (Hardware::CN, Hardware::BST),
                    _ => return None,
                }
            }
            Layer::GTPU => {
                // GTPU: BST <-> CN (uses TO/FROM like NGAP)
                match direction {
                    Direction::TO => (Hardware::BST, Hardware::CN),
                    Direction::FROM => (Hardware::CN, Hardware::BST),
                    _ => return None,
                }
            }
            _ => return None, // Other layers are ignored
        };
        
        Some(MessageArrow {
            trace_index: index,
            from,
            to,
            layer: trace.layer.clone(),
            message,
        })
    }
}

/// Chronograph Panel
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chronograph {
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
}

impl Default for Chronograph {
    fn default() -> Self {
        Self::new()
    }
}

impl Chronograph {
    /// Create a new Chronograph panel
    pub fn new() -> Self {
        Self {
            current_index: 0,
            arrows: Vec::new(),
            scroll_offset: 0.0,
            should_scroll: false,
            related_parent: None,
            related_child: None,
        }
    }
    
    /// Add arrow for current trace if it doesn't exist yet
    fn add_arrow_for_current(&mut self, data: &Data) {
        let current_idx = data.current_index;
        
        // Check if we already have an arrow for this index
        if self.arrows.iter().any(|a| a.trace_index == current_idx) {
            log::debug!("Arrow already exists for index {}", current_idx);
            return;
        }
        
        // Limit to 500 arrows - drop first 50 when reaching limit
        if self.arrows.len() >= 500 {
            log::debug!("Reached maximum of 500 arrows, dropping first 50");
            self.arrows.drain(0..50);
        }
        
        // Try to create arrow for current trace
        if let Some(trace) = data.events.get(current_idx) {
            if let Some(arrow) = MessageArrow::from_trace(trace, current_idx) {
                self.arrows.push(arrow);
                log::debug!("Added arrow for index {}, total now: {}", current_idx, self.arrows.len());
            } else {
                // log::debug!("No arrow created for index {} (layer not supported)", current_idx);
            }
        }
    }
    
    /// Get X position for a hardware element
    fn get_hardware_x(hardware: Hardware, rect: &Rect) -> f32 {
        let width = rect.width();
        let padding = width * 0.1; // 10% width
        match hardware {
            Hardware::UE => rect.left() + padding,
            Hardware::BST => rect.left() + width * 0.5,
            Hardware::CN => rect.right() - padding,
        }
    }
    
    /// Draw the chronograph
    fn draw_chronograph(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        
        // Use a scroll area for the timeline
        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false]);
        
        // Conditionally scroll to current arrow if flag is set
        let scroll_area = if self.should_scroll {
            if let Some(current_arrow_idx) = self.arrows.iter().position(|a| a.trace_index == self.current_index) {
                let arrow_height = 40.0;
                let header_height = 50.0;
                let arrow_y_position = (current_arrow_idx as f32 * arrow_height) + header_height + 10.0;
                
                // Center the arrow in the viewport by subtracting half the viewport height
                let viewport_height = available_size.y;
                let centered_offset = (arrow_y_position - viewport_height / 2.0).max(0.0);
                
                log::debug!("Chronograph: Scrolling to centered offset {} (arrow at index {})", centered_offset, self.current_index);
                scroll_area.vertical_scroll_offset(centered_offset)
            } else {
                scroll_area
            }
        } else {
            scroll_area
        };
        
        scroll_area.show(ui, |ui| {
                // Reserve space for all arrows + padding
                let arrow_height = 40.0;
                let total_height = self.arrows.len() as f32 * arrow_height + 100.0;
                let (rect, _response) = ui.allocate_exact_size(
                    Vec2::new(available_size.x - 20.0, total_height),
                    egui::Sense::hover()
                );
                
                let painter = ui.painter();
                
                // Add padding at the top for headers
                let header_height = 50.0;
                let arrow_start_y = rect.top() + header_height;
                
                // Draw vertical lines for hardware (starting after header)
                let line_color = Color32::from_rgb(100, 100, 100);
                let line_stroke = Stroke::new(2.0, line_color);
                
                let ue_x = Self::get_hardware_x(Hardware::UE, &rect);
                let bst_x = Self::get_hardware_x(Hardware::BST, &rect);
                let cn_x = Self::get_hardware_x(Hardware::CN, &rect);
                
                painter.line_segment(
                    [Pos2::new(ue_x, arrow_start_y), Pos2::new(ue_x, rect.bottom())],
                    line_stroke,
                );
                painter.line_segment(
                    [Pos2::new(bst_x, arrow_start_y), Pos2::new(bst_x, rect.bottom())],
                    line_stroke,
                );
                painter.line_segment(
                    [Pos2::new(cn_x, arrow_start_y), Pos2::new(cn_x, rect.bottom())],
                    line_stroke,
                );
                
                // Draw labels at the top
                painter.text(
                    Pos2::new(ue_x, rect.top() + 20.0),
                    egui::Align2::CENTER_CENTER,
                    "UE",
                    egui::FontId::proportional(16.0),
                    Color32::BLACK,
                );
                painter.text(
                    Pos2::new(bst_x, rect.top() + 20.0),
                    egui::Align2::CENTER_CENTER,
                    "BST",
                    egui::FontId::proportional(16.0),
                    Color32::BLACK,
                );
                painter.text(
                    Pos2::new(cn_x, rect.top() + 20.0),
                    egui::Align2::CENTER_CENTER,
                    "CN",
                    egui::FontId::proportional(16.0),
                    Color32::BLACK,
                );
                
                // Draw arrows
                let start_y = arrow_start_y + 10.0;
                    for (i, arrow) in self.arrows.iter().enumerate() {
                        let y = start_y + (i as f32 * arrow_height);
                        let is_current: bool = arrow.trace_index == self.current_index;
                        let is_related_parent = self.related_parent.as_ref().map_or(false, |v| v.contains(&arrow.trace_index));
                        let is_related_child = self.related_child.as_ref().map_or(false, |v| v.contains(&arrow.trace_index));
                        
                        // Determine arrow color and thickness
                        // Current: bright blue, Related (parent/child): lighter blue, Others: black
                        let (arrow_color, arrow_width) = if is_current {
                            (Color32::from_rgb(50, 120, 220), 3.0) // Blue and thicker for current
                        } else if is_related_parent || is_related_child {
                            (Color32::from_rgb(130, 180, 240), 2.5) // Lighter blue for related traces
                        } else {
                            (Color32::BLACK, 1.5) // Black for others
                        };
                        
                        let from_x = Self::get_hardware_x(arrow.from, &rect);
                        let to_x = Self::get_hardware_x(arrow.to, &rect);
                        
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
                        let text = format!("{:?} - {}", arrow.layer, arrow.message);
                        let text_pos = Pos2::new((from_x + to_x) / 2.0, y - 10.0);
                        painter.text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(10.0),
                            arrow_color,
                        );
                    }
            });
        
        // Reset scroll flag after drawing
        self.should_scroll = false;
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for Chronograph {
    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When a new event is added, create an arrow for it
        if let Some(arrow) = MessageArrow::from_trace(event, index) {
            // Check if arrow already exists (avoid duplicates when changing filters)
            if self.arrows.iter().any(|a| a.trace_index == index) {
                log::trace!("Chronograph: Arrow for event {} already exists, skipping", index);
                return;
            }
            
            // Find the correct position to insert based on trace_index
            // This ensures arrows are always in chronological order
            let insert_pos = self.arrows.iter()
                .position(|a| a.trace_index > index)
                .unwrap_or(self.arrows.len());
            
            log::trace!("Chronograph: Inserting arrow at position {} for event {} (layer: {:?})", 
                insert_pos, index, event.layer);
            
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
        
        log::trace!("Chronograph: Focused on event {}, parent={:?}, child={:?}", 
            index, self.related_parent, self.related_child);
    }
    
    fn on_events_cleared(&mut self) {
        log::debug!("Chronograph: Clearing all arrows");
        self.arrows.clear();
        self.current_index = 0;
        self.scroll_offset = 0.0;
        self.should_scroll = false;
        self.related_parent = None;
        self.related_child = None;
    }
    
    fn name(&self) -> &'static str {
        "Chronograph"
    }
    
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Chronograph")
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
impl super::PanelView for Chronograph {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Note: Data is not available here, so we show arrows without it
        // This is fine since EventSubscriber updates the arrows
        self.draw_chronograph(ui);
    }
}
