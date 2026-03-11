//! HARQ Panel
//!
//! Displays a chronograph-like timeline of PHY events (PDSCH/PUSCH),
//! colored by HARQ process number. The focused event arrow is highlighted.

use crate::event_system::{EventSubscriber, EventContext};
use egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::theme::ThemeColors;
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::types::Direction,
    interface::parser::parser_phy::PHYChannelType,
};

/// HARQ process color palette (16 colors for HARQ 0-15)
const HARQ_COLORS: [Color32; 16] = [
    Color32::from_rgb(230, 25, 75),    // 0  Red
    Color32::from_rgb(60, 180, 75),    // 1  Green
    Color32::from_rgb(0, 130, 200),    // 2  Blue
    Color32::from_rgb(255, 178, 29),   // 3  Orange
    Color32::from_rgb(145, 30, 180),   // 4  Purple
    Color32::from_rgb(70, 240, 240),   // 5  Cyan
    Color32::from_rgb(240, 50, 230),   // 6  Magenta
    Color32::from_rgb(210, 245, 60),   // 7  Lime
    Color32::from_rgb(250, 190, 212),  // 8  Pink
    Color32::from_rgb(0, 128, 128),    // 9  Teal
    Color32::from_rgb(220, 190, 255),  // 10 Lavender
    Color32::from_rgb(170, 110, 40),   // 11 Brown
    Color32::from_rgb(128, 0, 0),      // 12 Maroon
    Color32::from_rgb(170, 255, 195),  // 13 Mint
    Color32::from_rgb(128, 128, 0),    // 14 Olive
    Color32::from_rgb(255, 215, 180),  // 15 Apricot
];

/// Get the color for a HARQ process number
fn harq_color(harq: u8) -> Color32 {
    HARQ_COLORS[(harq as usize) % HARQ_COLORS.len()]
}

/// Arrow representation for a PHY event with HARQ info
#[derive(Debug, Clone)]
struct HarqArrow {
    /// Index in the trace list
    trace_index: usize,
    /// Direction (UL / DL)
    direction: Direction,
    /// Channel type (PDSCH / PUSCH)
    channel_type: PHYChannelType,
    /// HARQ process number
    harq: u8,
    /// Label text (e.g. "PDSCH harq=0 prb=2:47")
    label: String,
}

impl HarqArrow {
    /// Try to create a HarqArrow from a trace
    fn from_trace(trace: &Trace, index: usize) -> Option<Self> {
        let phy = match &trace.additional_infos {
            AdditionalInfos::PHYInfos(info) => info,
            _ => return None,
        };

        // Only keep PDSCH and PUSCH (the channels that carry HARQ)
        if !matches!(phy.channel_type, PHYChannelType::PDSCH | PHYChannelType::PUSCH) {
            return None;
        }

        let harq = phy.harq?;

        let label = format!(
            "{:?} harq={} prb={}:{} f={}.{}",
            phy.channel_type, harq, phy.prb_start, phy.prb_length, phy.frame, phy.slot,
        );

        Some(HarqArrow {
            trace_index: index,
            direction: phy.direction.clone(),
            channel_type: phy.channel_type,
            harq,
            label,
        })
    }
}

/// HARQ Panel — chronograph-style view of PHY HARQ events
#[derive(serde::Deserialize, serde::Serialize)]
pub struct HarqPanel {
    /// Current focused trace index
    #[serde(skip)]
    current_index: usize,

    /// Cached arrows
    #[serde(skip)]
    arrows: Vec<HarqArrow>,

    /// Flag to scroll to current arrow on next frame
    should_scroll: bool,
}

impl Default for HarqPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl HarqPanel {
    /// Create a new HARQ panel
    pub fn new() -> Self {
        Self {
            current_index: 0,
            arrows: Vec::new(),
            should_scroll: false,
        }
    }

    /// Get X position for UE (left) or BST (right)
    fn endpoint_x(is_ue: bool, rect: &Rect) -> f32 {
        let padding = rect.width() * 0.12;
        if is_ue {
            rect.left() + padding
        } else {
            rect.right() - padding
        }
    }

    /// Draw the HARQ chronograph
    fn draw_harq(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let arrow_height = 40.0_f32;
        let header_height = 50.0_f32;

        // Build scroll area, optionally scrolling to current arrow
        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false]);

        if self.should_scroll {
            if let Some(pos) = self.arrows.iter().position(|a| a.trace_index == self.current_index) {
                let arrow_y = (pos as f32 * arrow_height) + header_height + 10.0;
                let centered = (arrow_y - available_size.y / 2.0).max(0.0);
                scroll_area = scroll_area.vertical_scroll_offset(centered);
            }
        }

        scroll_area.show(ui, |ui| {
            let total_height = self.arrows.len() as f32 * arrow_height + 100.0;
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(available_size.x - 20.0, total_height),
                egui::Sense::hover(),
            );

            let painter = ui.painter();
            let theme = ThemeColors::get(ui);

            let ue_x = Self::endpoint_x(true, &rect);
            let bst_x = Self::endpoint_x(false, &rect);
            let arrow_start_y = rect.top() + header_height;

            // Vertical lifelines
            let line_stroke = Stroke::new(2.0, theme.text_weak);
            painter.line_segment(
                [Pos2::new(ue_x, arrow_start_y), Pos2::new(ue_x, rect.bottom())],
                line_stroke,
            );
            painter.line_segment(
                [Pos2::new(bst_x, arrow_start_y), Pos2::new(bst_x, rect.bottom())],
                line_stroke,
            );

            // Column headers
            painter.text(
                Pos2::new(ue_x, rect.top() + 20.0),
                egui::Align2::CENTER_CENTER,
                "UE",
                egui::FontId::proportional(16.0),
                theme.text,
            );
            painter.text(
                Pos2::new(bst_x, rect.top() + 20.0),
                egui::Align2::CENTER_CENTER,
                "BST",
                egui::FontId::proportional(16.0),
                theme.text,
            );

            // Draw arrows
            let start_y = arrow_start_y + 10.0;
            for (i, arrow) in self.arrows.iter().enumerate() {
                let y = start_y + (i as f32 * arrow_height);
                let is_current = arrow.trace_index == self.current_index;

                // Arrow direction: UL = UE→BST, DL = BST→UE
                let (from_x, to_x) = match arrow.direction {
                    Direction::UL => (ue_x, bst_x),
                    _ => (bst_x, ue_x),
                };

                // Color by HARQ process; brighten / thicken if focused
                let base_color = harq_color(arrow.harq);
                let (color, width) = if is_current {
                    (Color32::WHITE, 3.5)
                } else {
                    (base_color, 1.5)
                };

                // Draw arrow line
                painter.line_segment(
                    [Pos2::new(from_x, y), Pos2::new(to_x, y)],
                    Stroke::new(width, color),
                );

                // Arrowhead
                let arrow_size = 8.0;
                let dir = if to_x > from_x { 1.0 } else { -1.0 };
                let tip = Pos2::new(to_x, y);
                let base1 = Pos2::new(to_x - dir * arrow_size, y - arrow_size / 2.0);
                let base2 = Pos2::new(to_x - dir * arrow_size, y + arrow_size / 2.0);
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, base1, base2],
                    color,
                    Stroke::NONE,
                ));

                // If current, draw a colored highlight bar behind the arrow
                if is_current {
                    let highlight_rect = Rect::from_min_max(
                        Pos2::new(rect.left(), y - arrow_height / 2.0 + 2.0),
                        Pos2::new(rect.right(), y + arrow_height / 2.0 - 2.0),
                    );
                    painter.rect_filled(highlight_rect, 2.0, base_color.linear_multiply(0.18));
                }

                // HARQ color indicator dot
                let dot_x = from_x + (to_x - from_x).signum() * 14.0;
                painter.circle_filled(Pos2::new(dot_x, y), 5.0, base_color);

                // Label text above arrow
                let text_pos = Pos2::new((from_x + to_x) / 2.0, y - 10.0);
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    &arrow.label,
                    egui::FontId::proportional(10.0),
                    color,
                );
            }

            // Draw HARQ legend at the bottom of the visible area
            self.draw_legend(ui, &rect, &theme);
        });

        self.should_scroll = false;
    }

    /// Draw a compact HARQ color legend
    fn draw_legend(&self, ui: &mut egui::Ui, _rect: &Rect, _theme: &ThemeColors) {
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("HARQ: ");
            // Determine which HARQ IDs are actually present
            let mut seen: Vec<u8> = self.arrows.iter().map(|a| a.harq).collect();
            seen.sort();
            seen.dedup();
            for h in seen {
                let color = harq_color(h);
                let (response, painter) = ui.allocate_painter(Vec2::new(30.0, 16.0), egui::Sense::hover());
                let r = response.rect;
                painter.rect_filled(r, 3.0, color);
                painter.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", h),
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
            }
        });
    }
}

// ── EventSubscriber ──────────────────────────────────────────────────

impl EventSubscriber for HarqPanel {
    fn name(&self) -> &'static str {
        "HARQ"
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {}

    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        if let Some(arrow) = HarqArrow::from_trace(event, index) {
            if self.arrows.iter().any(|a| a.trace_index == index) {
                return;
            }

            let insert_pos = self.arrows.iter()
                .position(|a| a.trace_index > index)
                .unwrap_or(self.arrows.len());

            self.arrows.insert(insert_pos, arrow);

            if self.arrows.len() > 500 {
                self.arrows.remove(0);
            }
        }
    }

    fn on_event_focused(&mut self, _event: &Trace, index: usize, _context: &EventContext) {
        self.current_index = index;
        self.should_scroll = true;
    }

    fn on_events_cleared(&mut self) {
        self.arrows.clear();
        self.current_index = 0;
        self.should_scroll = false;
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("HARQ")
            .resizable(true)
            .default_width(640.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                self.draw_harq(ui);
            });
        Ok(())
    }
}

// ── PanelView ────────────────────────────────────────────────────────

impl super::PanelView for HarqPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.draw_harq(ui);
    }
}
