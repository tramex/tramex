//! Power Panel
//!
//! Displays three point graphs sharing the same SFN (System Frame Number) X axis,
//! built from the uplink measurements reported on PUSCH / PUCCH PHY traces:
//! EPRE (energy per resource element), TA (timing advance) and CSI.
//!
//! Only points are drawn (no lines) since events do not arrive in SFN order.
//! The X axis shows a window of frames which is re-centered on the focused event.

use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::NAVIGATE_REQUEST_ID;
use crate::panels::resources_blocks::ResourceType;
use crate::theme::ThemeColors;
use egui::{self, Color32, CursorIcon, PopupAnchor, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::parser::parser_phy::{PHYChannelData, PHYChannelType, UlMeasurements},
};

/// Number of frames per hyper frame
const FRAMES_PER_HFN: usize = 1024;

/// Number of slots per frame (2 slots/subframe × 10 subframes)
const SLOTS_PER_FRAME: usize = 20;

/// Default X window size in frames (1 frame = 10ms)
const DEFAULT_WINDOW_FRAMES: u16 = 10;

/// Width reserved on the left for Y axis labels
const Y_LABEL_WIDTH: f32 = 56.0;

/// Height reserved at the bottom for X axis labels
const X_LABEL_HEIGHT: f32 = 28.0;

/// Height of each graph title row
const GRAPH_TITLE_HEIGHT: f32 = 18.0;

/// Radius of a plotted point
const POINT_RADIUS: f32 = 3.0;

/// Max squared distance (px²) between the pointer and a point for hovering
const HOVER_DIST_SQ: f32 = 8.0 * 8.0;

/// Tooltip anchor
const TOOLTIP_ANCHOR: PopupAnchor = PopupAnchor::Pointer;

/// Compute the absolute slot index of a (hfn, frame, slot) triplet
fn global_slot(hfn: u32, frame: u16, slot: u8) -> usize {
    (hfn as usize * FRAMES_PER_HFN + frame as usize) * SLOTS_PER_FRAME + slot as usize
}

/// A measurement sample extracted from a PUSCH / PUCCH trace
#[derive(Debug, Clone)]
struct PowerSample {
    /// Index in the trace list
    trace_index: usize,
    /// Channel type (PUSCH or PUCCH)
    channel_type: PHYChannelType,
    /// Hyper Frame Number (for frame wrap handling)
    hfn: u32,
    /// System Frame Number
    frame: u16,
    /// Slot number within frame
    slot: u8,
    /// Parsed measurements
    measurements: UlMeasurements,
}

impl PowerSample {
    /// Try to create a sample from a trace (PUSCH / PUCCH with at least one measurement)
    fn from_trace(trace: &Trace, index: usize) -> Option<Self> {
        let AdditionalInfos::PHYInfos(phy) = &trace.additional_infos else {
            return None;
        };
        let measurements = match &phy.channel_data {
            PHYChannelData::Pusch { measurements, .. } | PHYChannelData::Pucch { measurements, .. } => measurements.clone(),
            _ => return None,
        };
        if measurements == UlMeasurements::default() {
            return None;
        }
        Some(Self {
            trace_index: index,
            channel_type: phy.channel_type,
            hfn: 0, // Set by PowerPanel during insertion
            frame: phy.frame,
            slot: phy.slot,
            measurements,
        })
    }

    /// Absolute slot index of this sample
    fn global_slot(&self) -> usize {
        global_slot(self.hfn, self.frame, self.slot)
    }
}

/// Which measurement a graph displays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Metric {
    /// Energy Per Resource Element
    Epre,
    /// Timing Advance
    Ta,
    /// Channel State Information
    Csi,
}

impl Metric {
    /// All metrics, in display order (top to bottom)
    const ALL: [Metric; 3] = [Metric::Epre, Metric::Ta, Metric::Csi];

    /// Graph title
    fn title(self) -> &'static str {
        match self {
            Metric::Epre => "EPRE (dB)",
            Metric::Ta => "Timing Advance (µs)",
            Metric::Csi => "CSI",
        }
    }

    /// Extract the metric value from measurements
    fn value(self, m: &UlMeasurements) -> Option<f64> {
        match self {
            Metric::Epre => m.epre.map(f64::from),
            Metric::Ta => m.ta.map(f64::from),
            Metric::Csi => m.csi.map(f64::from),
        }
    }

    /// Minimum Y span so that flat data is not zoomed into noise
    fn min_span(self) -> f64 {
        match self {
            Metric::Epre => 2.0,
            Metric::Ta => 1.0,
            Metric::Csi => 2.0,
        }
    }

    /// Format a value for axis labels / tooltips
    fn format(self, v: f64) -> String {
        match self {
            Metric::Epre | Metric::Ta => format!("{v:.1}"),
            Metric::Csi => format!("{v:.0}"),
        }
    }
}

/// Power Panel — EPRE / TA / CSI point graphs over SFN
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PowerPanel {
    /// Samples, sorted by absolute slot index
    #[serde(skip)]
    samples: Vec<PowerSample>,

    /// Start of the X window as absolute slot index
    #[serde(skip)]
    start_slot: usize,

    /// X window size in frames
    window_frames: u16,

    /// Show PUSCH samples
    show_pusch: bool,

    /// Show PUCCH samples
    show_pucch: bool,

    /// Currently focused trace index (for highlight)
    #[serde(skip)]
    focused_trace_index: Option<usize>,

    /// HFN tracking: (hfn, frame, slot) of the latest event seen
    #[serde(skip)]
    end_limit: (u32, u16, u8),

    /// Whether we have received any event yet
    #[serde(skip)]
    has_first_event: bool,

    /// Sub-slot drag remainder, so slow drags still pan
    #[serde(skip)]
    drag_remainder: f32,
}

impl Default for PowerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerPanel {
    /// Create a new Power panel
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            start_slot: 0,
            window_frames: DEFAULT_WINDOW_FRAMES,
            show_pusch: true,
            show_pucch: true,
            focused_trace_index: None,
            end_limit: (0, 0, 0),
            has_first_event: false,
            drag_remainder: 0.0,
        }
    }

    /// Total number of slots in the display window
    fn window_total_slots(&self) -> usize {
        self.window_frames.max(1) as usize * SLOTS_PER_FRAME
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
        let end_frame = self.end_limit.1;

        if candidate_pos > end_pos {
            // Numerically ahead: either newer, or from the previous HFN (anti-wrap)
            if frame > end_frame && (frame - end_frame) > 512 {
                hfn = hfn.saturating_sub(1);
            } else {
                self.end_limit = (hfn, frame, slot);
            }
        } else if candidate_pos < end_pos && frame < end_frame && (end_frame - frame) > 512 {
            // Frame wrapped around (e.g. 1023 -> 0): new HFN
            hfn += 1;
            self.end_limit = (hfn, frame, slot);
        }

        hfn
    }

    /// Best-effort HFN for a focused PHY event that may not be a sample itself
    fn hfn_for(&self, trace_index: usize, frame: u16, slot: u8) -> u32 {
        self.samples
            .iter()
            .find(|s| s.trace_index == trace_index)
            .or_else(|| self.samples.iter().find(|s| s.frame == frame && s.slot == slot))
            .or_else(|| self.samples.iter().min_by_key(|s| s.trace_index.abs_diff(trace_index)))
            .map_or(self.end_limit.0, |s| s.hfn)
    }

    /// Center the X window on an absolute slot index
    fn center_on(&mut self, slot: usize) {
        self.start_slot = slot.saturating_sub(self.window_total_slots() / 2);
    }

    /// Shift the X window by a signed number of slots
    fn shift_slots(&mut self, delta: isize) {
        self.start_slot = (self.start_slot as isize + delta).max(0) as usize;
    }

    /// Point color for a channel type
    fn channel_color(channel: PHYChannelType, is_dark: bool) -> Color32 {
        match channel {
            PHYChannelType::PUCCH => ResourceType::Pucch.color(is_dark),
            _ => ResourceType::Pusch.color(is_dark),
        }
    }

    /// Whether a sample passes the channel filter
    fn is_shown(&self, sample: &PowerSample) -> bool {
        match sample.channel_type {
            PHYChannelType::PUSCH => self.show_pusch,
            PHYChannelType::PUCCH => self.show_pucch,
            _ => false,
        }
    }

    /// Control bar: navigation, window size, channel filter
    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        let is_dark = ui.visuals().dark_mode;
        ui.horizontal_wrapped(|ui| {
            ui.label("Window (frames):");
            ui.add(egui::Slider::new(&mut self.window_frames, 5..=500).logarithmic(true));

            ui.separator();

            let spf = SLOTS_PER_FRAME as isize;
            if ui.button("◀◀").clicked() {
                self.shift_slots(-spf);
            }
            let sf = self.start_slot / SLOTS_PER_FRAME % FRAMES_PER_HFN;
            let ef = (self.start_slot + self.window_total_slots() - 1) / SLOTS_PER_FRAME % FRAMES_PER_HFN;
            ui.label(format!("Frames {}-{}", sf, ef));
            if ui.button("▶▶").clicked() {
                self.shift_slots(spf);
            }

            ui.separator();

            for (label, rt, flag) in [
                ("PUSCH", ResourceType::Pusch, &mut self.show_pusch),
                ("PUCCH", ResourceType::Pucch, &mut self.show_pucch),
            ] {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                let color = if *flag {
                    rt.color(is_dark)
                } else {
                    rt.color(is_dark).linear_multiply(0.3)
                };
                ui.painter().rect_filled(rect, 2.0, color);
                ui.checkbox(flag, egui::RichText::new(label).small());
            }
        });
    }

    /// Draw the three graphs
    fn draw_power(&mut self, ui: &mut egui::Ui) {
        self.draw_controls(ui);
        ui.separator();

        let theme = ThemeColors::get(ui);
        let is_dark = ui.visuals().dark_mode;
        let available = ui.available_size();
        if available.x < Y_LABEL_WIDTH + 40.0 || available.y < X_LABEL_HEIGHT + 3.0 * (GRAPH_TITLE_HEIGHT + 20.0) {
            return;
        }

        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        let plot_left = rect.left() + Y_LABEL_WIDTH;
        let plot_right = rect.right() - 8.0;
        let plot_width = plot_right - plot_left;
        let plots_top = rect.top();
        let plots_bottom = rect.bottom() - X_LABEL_HEIGHT;
        let graph_height = (plots_bottom - plots_top) / Metric::ALL.len() as f32;

        // Drag to pan the window
        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            let window_slots = self.window_total_slots() as f32;
            let delta = -response.drag_delta().x / plot_width * window_slots + self.drag_remainder;
            let whole = delta.trunc();
            self.drag_remainder = delta - whole;
            self.shift_slots(whole as isize);
        }

        let start_slot = self.start_slot;
        let window_slots = self.window_total_slots();
        let end_slot = start_slot + window_slots;
        let x_of = |g: usize| -> f32 { plot_left + (g - start_slot) as f32 / window_slots as f32 * plot_width };

        // Visible samples (list is sorted by absolute slot)
        let lo = self.samples.partition_point(|s| s.global_slot() < start_slot);
        let hi = self.samples.partition_point(|s| s.global_slot() < end_slot);
        let visible: Vec<&PowerSample> = self.samples[lo..hi].iter().filter(|s| self.is_shown(s)).collect();

        // Focused sample X position (if it is visible)
        let focused_x = self
            .focused_trace_index
            .and_then(|idx| visible.iter().find(|s| s.trace_index == idx))
            .map(|s| x_of(s.global_slot()));

        // Frame boundaries within the window, shared by every graph and the X axis
        let first_boundary = start_slot.div_ceil(SLOTS_PER_FRAME) * SLOTS_PER_FRAME;
        let boundaries: Vec<usize> = (first_boundary..end_slot).step_by(SLOTS_PER_FRAME).collect();

        let pointer_pos = response.hover_pos();
        let mut hovered: Option<(&PowerSample, Metric)> = None;
        let mut hovered_dist_sq = HOVER_DIST_SQ;

        for (i, metric) in Metric::ALL.iter().copied().enumerate() {
            let top = plots_top + i as f32 * graph_height;
            let graph_rect = Rect::from_min_max(
                Pos2::new(plot_left, top + GRAPH_TITLE_HEIGHT),
                Pos2::new(plot_right, top + graph_height - 6.0),
            );

            painter.text(
                Pos2::new(plot_left + 4.0, top + GRAPH_TITLE_HEIGHT / 2.0),
                egui::Align2::LEFT_CENTER,
                metric.title(),
                egui::FontId::proportional(12.0),
                theme.text_strong,
            );
            painter.rect_stroke(graph_rect, 2.0, Stroke::new(1.0, theme.line_medium), StrokeKind::Inside);

            // Vertical frame boundaries
            for &g in &boundaries {
                let x = x_of(g);
                let is_hfn_boundary = (g / SLOTS_PER_FRAME).is_multiple_of(FRAMES_PER_HFN);
                let color = if is_hfn_boundary {
                    theme.line_strong
                } else {
                    theme.line_small
                };
                painter.line_segment(
                    [Pos2::new(x, graph_rect.top()), Pos2::new(x, graph_rect.bottom())],
                    Stroke::new(if is_hfn_boundary { 1.5 } else { 0.5 }, color),
                );
            }

            // Y range from visible values
            let (min, max) = visible
                .iter()
                .filter_map(|s| metric.value(&s.measurements))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)));
            if min > max {
                painter.text(
                    graph_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No data in window",
                    egui::FontId::proportional(12.0),
                    theme.text_weak,
                );
                continue;
            }
            let span = (max - min).max(metric.min_span());
            let pad = span * 0.1;
            let (y_min, y_max) = ((min + max - span) / 2.0 - pad, (min + max + span) / 2.0 + pad);
            let y_of =
                |v: f64| -> f32 { graph_rect.bottom() - ((v - y_min) / (y_max - y_min)) as f32 * graph_rect.height() };

            // Horizontal grid + Y labels
            const Y_TICKS: usize = 4;
            for k in 0..=Y_TICKS {
                let v = y_min + (y_max - y_min) * k as f64 / Y_TICKS as f64;
                let y = y_of(v);
                painter.line_segment(
                    [Pos2::new(graph_rect.left(), y), Pos2::new(graph_rect.right(), y)],
                    Stroke::new(0.5, theme.line_small),
                );
                painter.text(
                    Pos2::new(plot_left - 4.0, y),
                    egui::Align2::RIGHT_CENTER,
                    metric.format(v),
                    egui::FontId::proportional(9.0),
                    theme.text_weak,
                );
            }

            // Focused event vertical marker
            if let Some(x) = focused_x {
                painter.line_segment(
                    [Pos2::new(x, graph_rect.top()), Pos2::new(x, graph_rect.bottom())],
                    Stroke::new(1.0, theme.accent),
                );
            }

            // Points
            for sample in &visible {
                let Some(v) = metric.value(&sample.measurements) else {
                    continue;
                };
                let pos = Pos2::new(x_of(sample.global_slot()), y_of(v));
                let color = Self::channel_color(sample.channel_type, is_dark);
                let is_focused = self.focused_trace_index == Some(sample.trace_index);

                if is_focused {
                    painter.circle(pos, POINT_RADIUS + 3.0, color, Stroke::new(2.0, theme.accent));
                } else {
                    painter.circle_filled(pos, POINT_RADIUS, color);
                }

                if let Some(p) = pointer_pos {
                    let d = pos.distance_sq(p);
                    if d < hovered_dist_sq {
                        hovered_dist_sq = d;
                        hovered = Some((sample, metric));
                    }
                }
            }
        }

        // X axis labels: frame numbers at frame boundaries, thinned to avoid overlap
        let max_labels = (plot_width / 44.0).max(1.0) as usize;
        let step = boundaries.len().div_ceil(max_labels).max(1);
        let label_y = plots_bottom + X_LABEL_HEIGHT / 2.0;
        for &g in boundaries.iter().step_by(step) {
            let frame = g / SLOTS_PER_FRAME % FRAMES_PER_HFN;
            painter.text(
                Pos2::new(x_of(g), label_y),
                egui::Align2::CENTER_CENTER,
                frame.to_string(),
                egui::FontId::proportional(10.0),
                theme.text,
            );
        }
        painter.text(
            Pos2::new(plot_left - 4.0, label_y),
            egui::Align2::RIGHT_CENTER,
            "SFN",
            egui::FontId::proportional(10.0),
            theme.text_weak,
        );

        // Hover tooltip + click-to-navigate
        if let Some((sample, metric)) = hovered {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            let channel = match sample.channel_type {
                PHYChannelType::PUCCH => "PUCCH",
                _ => "PUSCH",
            };
            let m = &sample.measurements;
            egui::Tooltip::always_open(
                ui.ctx().clone(),
                ui.layer_id(),
                egui::Id::new("power_tooltip"),
                TOOLTIP_ANCHOR,
            )
            .at_pointer()
            .show(|ui| {
                ui.label(egui::RichText::new(format!("{} — Frame {} Slot {}", channel, sample.frame, sample.slot)).strong());
                for other in Metric::ALL {
                    if let Some(v) = other.value(m) {
                        let text = format!("{}: {}", other.title(), other.format(v));
                        if other == metric {
                            ui.label(egui::RichText::new(text).strong());
                        } else {
                            ui.label(text);
                        }
                    }
                }
                if let Some(csi) = m.csi {
                    ui.label(egui::RichText::new(format!("CSI bits: {:b}", csi)).small());
                }
            });
            if response.clicked() {
                ui.ctx()
                    .data_mut(|d| d.insert_temp(egui::Id::new(NAVIGATE_REQUEST_ID), sample.trace_index));
            }
        } else if response.hovered() && !response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
    }
}

// ── EventSubscriber ──────────────────────────────────────────────────

impl EventSubscriber for PowerPanel {
    fn name(&self) -> &'static str {
        "Power"
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {}

    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        let Some(mut sample) = PowerSample::from_trace(event, index) else {
            return;
        };
        sample.hfn = self.compute_hfn(sample.frame, sample.slot);
        let key = sample.global_slot();
        let pos = self.samples.partition_point(|s| s.global_slot() <= key);
        self.samples.insert(pos, sample);
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        self.focused_trace_index = Some(index);

        // Center the window on the focused event: use its own SFN if it is a PHY
        // trace, otherwise the closest sample (by trace index) as a proxy.
        let target = match &event.additional_infos {
            AdditionalInfos::PHYInfos(phy) => {
                Some(global_slot(self.hfn_for(index, phy.frame, phy.slot), phy.frame, phy.slot))
            }
            _ => self
                .samples
                .iter()
                .min_by_key(|s| s.trace_index.abs_diff(index))
                .map(PowerSample::global_slot),
        };
        if let Some(slot) = target {
            self.center_on(slot);
        }
    }

    fn on_events_cleared(&mut self) {
        self.samples.clear();
        self.start_slot = 0;
        self.focused_trace_index = None;
        self.end_limit = (0, 0, 0);
        self.has_first_event = false;
        self.drag_remainder = 0.0;
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Power")
            .resizable(true)
            .default_width(720.0)
            .default_height(520.0)
            .open(open)
            .show(ctx, |ui| {
                self.draw_power(ui);
            });
        Ok(())
    }
}

// ── PanelView ────────────────────────────────────────────────────────

impl super::PanelView for PowerPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.draw_power(ui);
    }
}
