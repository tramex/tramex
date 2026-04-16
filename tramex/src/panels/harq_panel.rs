//! HARQ Panel
//!
//! Displays a chronograph-like timeline of PHY events (PDCCH, PDSCH, PUSCH, PUCCH),
//! colored by HARQ process number. The focused event arrow is highlighted.

use crate::event_system::{EventContext, EventSubscriber};
use crate::theme::ThemeColors;
use egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::parser::parser_phy::{PHYChannelData, PHYChannelType},
    interface::types::Direction,
};

const MAX_ARROWS: usize = 200;

/// HARQ process color palette (16 colors for HARQ 0-15)
const HARQ_COLORS: [Color32; 16] = [
    Color32::from_rgb(230, 25, 75),   // 0  Red
    Color32::from_rgb(60, 180, 75),   // 1  Green
    Color32::from_rgb(0, 130, 200),   // 2  Blue
    Color32::from_rgb(255, 178, 29),  // 3  Orange
    Color32::from_rgb(145, 30, 180),  // 4  Purple
    Color32::from_rgb(70, 240, 240),  // 5  Cyan
    Color32::from_rgb(240, 50, 230),  // 6  Magenta
    Color32::from_rgb(210, 245, 60),  // 7  Lime
    Color32::from_rgb(250, 190, 212), // 8  Pink
    Color32::from_rgb(0, 128, 128),   // 9  Teal
    Color32::from_rgb(220, 190, 255), // 10 Lavender
    Color32::from_rgb(170, 110, 40),  // 11 Brown
    Color32::from_rgb(128, 0, 0),     // 12 Maroon
    Color32::from_rgb(170, 255, 195), // 13 Mint
    Color32::from_rgb(128, 128, 0),   // 14 Olive
    Color32::from_rgb(255, 215, 180), // 15 Apricot
];

/// Get the color for a HARQ process number
fn harq_color(harq: u8) -> Color32 {
    HARQ_COLORS[(harq as usize) % HARQ_COLORS.len()]
}

/// Sort key for chronological ordering: (HFN, frame, slot)
type SfnKey = (u32, u16, u8);

/// Arrow representation for a PHY event with HARQ info
#[derive(Debug, Clone)]
struct HarqArrow {
    /// Index in the trace list
    trace_index: usize,
    /// Direction (UL / DL)
    direction: Direction,
    /// Channel type
    _channel_type: PHYChannelType,
    /// HARQ process number (None for PUCCH which has no direct harq id)
    harq: Option<u8>,
    /// Label text displayed on the arrow
    label: String,
    /// Hyper Frame Number (for frame wrap handling)
    hfn: u32,
    /// System Frame Number
    frame: u16,
    /// Slot number within frame
    slot: u8,
}

/// Format helper: display Option<u8> as value or "-"
fn fmt_opt(v: Option<u8>) -> String {
    v.map_or("-".to_string(), |v| v.to_string())
}

impl HarqArrow {
    /// Try to create a HarqArrow from a trace
    fn from_trace(trace: &Trace, index: usize) -> Option<Self> {
        let phy = match &trace.additional_infos {
            AdditionalInfos::PHYInfos(info) => info,
            _ => return None,
        };

        // Skip harq=si events (MIB/SIB carry)
        if phy.harq_si {
            return None;
        }

        // Only keep PDCCH, PDSCH, PUSCH, PUCCH
        if !matches!(
            phy.channel_type,
            PHYChannelType::PDCCH | PHYChannelType::PDSCH | PHYChannelType::PUSCH | PHYChannelType::PUCCH
        ) {
            return None;
        }

        let (harq, label) = match &phy.channel_data {
            PHYChannelData::Pdcch {
                dci,
                harq_process,
                ndi,
                rv_idx,
                ..
            } => {
                // Skip PDCCH without harq_process (e.g. DCI 1_0 for SIB)
                if harq_process.is_none() {
                    return None;
                }
                let label = format!(
                    "{}:{} PDCCH dci={} ndi={} rv_idx={}",
                    phy.frame,
                    phy.slot,
                    dci,
                    fmt_opt(*ndi),
                    fmt_opt(*rv_idx)
                );
                (*harq_process, label)
            }
            PHYChannelData::Pdsch { retx, rv_idx } => {
                let label = format!(
                    "{}:{} PDSCH retx={} rv_idx={}",
                    phy.frame,
                    phy.slot,
                    fmt_opt(*retx),
                    fmt_opt(*rv_idx)
                );
                (phy.harq, label)
            }
            PHYChannelData::Pusch { retx, rv_idx, crc } => {
                let crc_str = crc.map_or("-", |v| if v { "OK" } else { "KO" });
                let label = format!(
                    "{}:{} PUSCH retx={} rv_idx={} crc={}",
                    phy.frame,
                    phy.slot,
                    fmt_opt(*retx),
                    fmt_opt(*rv_idx),
                    crc_str
                );
                (phy.harq, label)
            }
            PHYChannelData::Pucch { format, ack } => {
                // Skip format=2 (CSI only, no HARQ feedback)
                if *format == Some(2) {
                    return None;
                }
                let ack_str = ack.map_or("-".to_string(), |v| if v { "ACK".to_string() } else { "NACK".to_string() });
                let label = format!("{}:{} PUCCH format={} {}", phy.frame, phy.slot, fmt_opt(*format), ack_str);
                (None, label) // No HARQ process on PUCCH
            }
            PHYChannelData::None => return None,
        };

        Some(HarqArrow {
            trace_index: index,
            direction: phy.direction.clone(),
            _channel_type: phy.channel_type,
            harq,
            label,
            hfn: 0, // Will be set by HarqPanel during insertion
            frame: phy.frame,
            slot: phy.slot,
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

    /// HFN tracking: (hfn, frame, slot) of the latest event seen
    #[serde(skip)]
    end_limit: (u32, u16, u8),

    /// Whether we have received any event yet
    #[serde(skip)]
    has_first_event: bool,
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
            end_limit: (0, 0, 0),
            has_first_event: false,
        }
    }

    /// Compute the HFN for a new event, handling frame wraparound (0..1023)
    fn compute_hfn(&mut self, frame: u16, slot: u8) -> u32 {
        if !self.has_first_event {
            self.has_first_event = true;
            self.end_limit = (0, frame, slot);
            return 0;
        }

        let mut hfn = self.end_limit.0;
        let candidate_pos = (frame, slot);
        let end_pos = (self.end_limit.1, self.end_limit.2);

        if candidate_pos > end_pos {
            // Candidate is numerically ahead of end_limit
            // Check for anti-wrap: frame jumped far ahead → actually from previous HFN
            let end_frame = self.end_limit.1;
            if frame > end_frame && (frame - end_frame) > 512 {
                hfn = hfn.saturating_sub(1);
            } else {
                self.end_limit = (hfn, frame, slot);
            }
        } else if candidate_pos < end_pos {
            // Candidate is numerically behind end_limit
            // Check for wrap: frame went from ~1023 to ~0 → new HFN
            let end_frame = self.end_limit.1;
            if frame < end_frame && (end_frame - frame) > 512 {
                hfn += 1;
                self.end_limit = (hfn, frame, slot);
            }
        }

        hfn
    }

    /// Draw the HARQ chronograph
    fn draw_harq(&mut self, ui: &mut egui::Ui) {
        // ── Fixed legend ──
        self.draw_legend(ui);

        let available_width = ui.available_width();
        let arrow_height = 40.0_f32;
        let theme = ThemeColors::get(ui);

        // Calculate UE/BST X positions based on available width
        let padding = available_width * 0.12;
        let ue_x = padding;
        let bst_x = available_width - padding;

        // ── Fixed header: UE / BST labels ──
        let header_rect = ui.allocate_space(Vec2::new(available_width, 30.0)).1;
        let painter = ui.painter();
        painter.text(
            Pos2::new(header_rect.left() + ue_x, header_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "UE",
            egui::FontId::proportional(16.0),
            theme.text,
        );
        painter.text(
            Pos2::new(header_rect.left() + bst_x, header_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "BST",
            egui::FontId::proportional(16.0),
            theme.text,
        );

        ui.separator();

        // ── Scrollable arrow area ──
        let scroll_height = ui.available_height();
        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);

        if self.should_scroll
            && let Some(pos) = self.arrows.iter().position(|a| a.trace_index == self.current_index) {
                let arrow_y = pos as f32 * arrow_height;
                let centered = (arrow_y - scroll_height / 2.0).max(0.0);
                scroll_area = scroll_area.vertical_scroll_offset(centered);
            }

        scroll_area.show(ui, |ui| {
            let total_height = (self.arrows.len() as f32 * arrow_height).max(100.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(available_width - 20.0, total_height), egui::Sense::hover());

            let painter = ui.painter();

            // Recalculate X positions relative to scroll area rect
            let ue_x = rect.left() + padding;
            let bst_x = rect.right() - padding;

            // Vertical lifelines
            let line_stroke = Stroke::new(2.0, theme.text_weak);
            painter.line_segment([Pos2::new(ue_x, rect.top()), Pos2::new(ue_x, rect.bottom())], line_stroke);
            painter.line_segment([Pos2::new(bst_x, rect.top()), Pos2::new(bst_x, rect.bottom())], line_stroke);

            // Draw arrows
            for (i, arrow) in self.arrows.iter().enumerate() {
                let y = rect.top() + (i as f32 * arrow_height) + arrow_height / 2.0;
                let is_current = arrow.trace_index == self.current_index;

                // Arrow direction: UL = UE→BST, DL = BST→UE
                let (from_x, to_x) = match arrow.direction {
                    Direction::UL => (ue_x, bst_x),
                    _ => (bst_x, ue_x),
                };

                // Color by HARQ process
                let base_color = arrow.harq.map(harq_color).unwrap_or(theme.text_weak); // PUCCH: neutral color
                let is_dark = ui.visuals().dark_mode;

                // Focused: highlight bar + contrasting arrow/text
                // Normal: harq-colored arrow, theme-aware label text
                let (arrow_color, arrow_width, label_color) = if is_current {
                    let highlight_bg = if is_dark {
                        base_color.linear_multiply(0.3)
                    } else {
                        base_color.linear_multiply(0.15)
                    };
                    let highlight_rect = Rect::from_min_max(
                        Pos2::new(rect.left(), y - arrow_height / 2.0 + 2.0),
                        Pos2::new(rect.right(), y + arrow_height / 2.0 - 2.0),
                    );
                    painter.rect_filled(highlight_rect, 2.0, highlight_bg);
                    // Use strong text for focused label, bright arrow
                    (theme.text_strong, 3.0, theme.text_strong)
                } else {
                    (base_color, 1.5, theme.text)
                };

                // Draw arrow line
                painter.line_segment(
                    [Pos2::new(from_x, y), Pos2::new(to_x, y)],
                    Stroke::new(arrow_width, arrow_color),
                );

                // Arrowhead
                let arrow_size = 8.0;
                let dir = if to_x > from_x { 1.0 } else { -1.0 };
                let tip = Pos2::new(to_x, y);
                let base1 = Pos2::new(to_x - dir * arrow_size, y - arrow_size / 2.0);
                let base2 = Pos2::new(to_x - dir * arrow_size, y + arrow_size / 2.0);
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, base1, base2],
                    arrow_color,
                    Stroke::NONE,
                ));

                // Label text above arrow
                let text_pos = Pos2::new((from_x + to_x) / 2.0, y - 10.0);
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    &arrow.label,
                    egui::FontId::proportional(10.0),
                    label_color,
                );
            }
        });

        self.should_scroll = false;
    }

    /// Draw a compact HARQ color legend
    fn draw_legend(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("HARQ: ");
            // Determine which HARQ IDs are actually present
            let mut seen: Vec<u8> = self.arrows.iter().filter_map(|a| a.harq).collect();
            seen.sort();
            seen.dedup();
            for h in seen {
                let color = harq_color(h);
                let (response, painter) = ui.allocate_painter(Vec2::new(30.0, 16.0), egui::Sense::hover());
                let r = response.rect;
                painter.rect_filled(r, 3.0, color);
                // Use theme text color for dark mode compatibility
                let text_color = ui.visuals().strong_text_color();
                painter.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", h),
                    egui::FontId::proportional(10.0),
                    text_color,
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
        if let Some(mut arrow) = HarqArrow::from_trace(event, index) {
            if self.arrows.iter().any(|a| a.trace_index == index) {
                return;
            }

            // Compute HFN for this arrow using anti-wrap logic
            arrow.hfn = self.compute_hfn(arrow.frame, arrow.slot);

            // Insert sorted by (hfn, frame, slot)
            let key: SfnKey = (arrow.hfn, arrow.frame, arrow.slot);
            let insert_pos = self
                .arrows
                .iter()
                .position(|a| (a.hfn, a.frame, a.slot) > key)
                .unwrap_or(self.arrows.len());

            self.arrows.insert(insert_pos, arrow);

            // Limit arrow history — drop first 50 when reaching limit
            if self.arrows.len() >= MAX_ARROWS {
                self.arrows.drain(0..50);
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
        self.end_limit = (0, 0, 0);
        self.has_first_event = false;
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
