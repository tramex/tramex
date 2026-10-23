//! GTPU Tunnel Panel
//!
//! Displays the GTPU tunnel between the BST and the remote UPF endpoint,
//! with the TEID and QFI of each direction.

use crate::event_system::{EventContext, EventSubscriber, PanelGroup};
use crate::panels::NAVIGATE_REQUEST_ID;
use crate::theme::{ArrowColors, ThemeColors};
use egui::{self, Color32, CursorIcon, Pos2, Rect, Sense, Stroke, Vec2};
use tramex_tools::{
    data::{AdditionalInfos, Trace},
    errors::TramexError,
    interface::{layer::Layer, parser::parser_gtpu::GTPUInfos, types::Direction},
};

/// Identifies a tunnel arrow: direction (uplink or not) and TEID
type TunnelKey = (bool, Option<u32>);

/// One arrow of the tunnel (one direction of one TEID)
#[derive(Debug, Clone)]
struct TunnelArrow {
    /// Uplink (BST -> UPF) or downlink (UPF -> BST)
    uplink: bool,
    /// Trace index of the first message seen on this TEID
    first_index: usize,
    /// Trace index of the latest message seen on this TEID
    last_index: usize,
    /// GTPU infos of the latest message
    infos: GTPUInfos,
}

impl TunnelArrow {
    /// Key of this arrow
    fn key(&self) -> TunnelKey {
        (self.uplink, self.infos.teid)
    }
}

/// Height of one arrow row
const ROW_HEIGHT: f32 = 45.0;
/// Height of the header (lifeline titles)
const HEADER_HEIGHT: f32 = 45.0;

/// GTPU Tunnel Panel
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct GtpuTunnel {
    /// One arrow per (direction, TEID), ordered by first appearance
    #[serde(skip)]
    tunnels: Vec<TunnelArrow>,

    /// Key of the tunnel of the currently focused trace
    #[serde(skip)]
    current: Option<TunnelKey>,
}

/// Whether a GTPU direction is uplink (BST -> core)
fn is_uplink(direction: &Direction) -> bool {
    matches!(direction, Direction::TO | Direction::UL)
}

/// Format the TEID as hexadecimal
fn format_teid(infos: &GTPUInfos) -> String {
    infos.teid.map_or_else(|| "?".to_owned(), |t| format!("0x{t:08x}"))
}

/// Format the QFI
fn format_qfi(infos: &GTPUInfos) -> String {
    infos.qfi.map_or_else(|| "?".to_owned(), |q| q.to_string())
}

impl GtpuTunnel {
    /// Create a new GTPU Tunnel panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a GTPU trace: update its tunnel arrow, or create one for a new TEID.
    /// Returns the key of the tunnel if the trace is a GTPU message.
    fn update(&mut self, event: &Trace, index: usize) -> Option<TunnelKey> {
        if event.layer != Layer::GTPU {
            return None;
        }
        let AdditionalInfos::GTPUInfos(infos) = &event.additional_infos else {
            return None;
        };
        let key = (is_uplink(&infos.direction), infos.teid);

        match self.tunnels.iter_mut().find(|t| t.key() == key) {
            Some(tunnel) => {
                tunnel.first_index = tunnel.first_index.min(index);
                if index >= tunnel.last_index {
                    tunnel.last_index = index;
                    tunnel.infos = infos.clone();
                }
            }
            None => {
                let pos = self
                    .tunnels
                    .iter()
                    .position(|t| t.first_index > index)
                    .unwrap_or(self.tunnels.len());
                self.tunnels.insert(
                    pos,
                    TunnelArrow {
                        uplink: key.0,
                        first_index: index,
                        last_index: index,
                        infos: infos.clone(),
                    },
                );
            }
        }
        Some(key)
    }

    /// Most relevant tunnel to describe the endpoints (focused one first, then most recent)
    fn reference(&self) -> Option<&TunnelArrow> {
        self.tunnels
            .iter()
            .find(|t| Some(t.key()) == self.current)
            .or_else(|| self.tunnels.iter().max_by_key(|t| t.last_index))
    }

    /// Draw one tunnel arrow. Returns true if clicked.
    #[allow(clippy::too_many_arguments)]
    fn draw_arrow(
        &self,
        ui: &mut egui::Ui,
        row: usize,
        rect: &Rect,
        from_x: f32,
        to_x: f32,
        y: f32,
        tunnel: &TunnelArrow,
        theme: &ThemeColors,
    ) -> bool {
        let (color, width) = if Some(tunnel.key()) == self.current {
            (ArrowColors::CURRENT, 3.0)
        } else {
            (theme.text, 1.5)
        };

        let row_rect = Rect::from_min_max(
            Pos2::new(rect.left(), y - ROW_HEIGHT / 2.0 - 5.0),
            Pos2::new(rect.right(), y + ROW_HEIGHT / 2.0 - 5.0),
        );
        let response = ui.interact(row_rect, egui::Id::new("gtpu_tunnel_arrow").with(row), Sense::click());
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        let painter = ui.painter();
        painter.line_segment([Pos2::new(from_x, y), Pos2::new(to_x, y)], Stroke::new(width, color));
        let size = 8.0;
        let sign = if to_x > from_x { 1.0 } else { -1.0 };
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(to_x, y),
                Pos2::new(to_x - sign * size, y - size / 2.0),
                Pos2::new(to_x - sign * size, y + size / 2.0),
            ],
            color,
            Stroke::NONE,
        ));

        let label = format!(
            "{}  TEID={}  QFI={}",
            if tunnel.uplink { "UL" } else { "DL" },
            format_teid(&tunnel.infos),
            format_qfi(&tunnel.infos)
        );
        painter.text(
            Pos2::new((from_x + to_x) / 2.0, y - 12.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(12.0),
            color,
        );

        response.clicked()
    }

    /// Draw the tunnel diagram
    fn draw(&mut self, ui: &mut egui::Ui) {
        if self.tunnels.is_empty() {
            ui.label("No GTPU message received yet.");
            return;
        }

        let theme = ThemeColors::get(ui);
        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
            let width = ui.available_width().max(300.0);
            let height = HEADER_HEIGHT + self.tunnels.len() as f32 * ROW_HEIGHT + 20.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

            let padding = width * 0.12;
            let bst_x = rect.left() + padding;
            let upf_x = rect.right() - padding;
            let top = rect.top() + HEADER_HEIGHT;

            let connection_info = self
                .reference()
                .and_then(|r| r.infos.connection_info.clone())
                .unwrap_or_default();
            {
                let painter = ui.painter();
                let line_stroke = Stroke::new(2.0, theme.text_weak);
                painter.line_segment([Pos2::new(bst_x, top), Pos2::new(bst_x, rect.bottom())], line_stroke);
                painter.line_segment([Pos2::new(upf_x, top), Pos2::new(upf_x, rect.bottom())], line_stroke);

                for (x, title, subtitle) in [(bst_x, "BST", String::new()), (upf_x, "UPF", connection_info)] {
                    painter.text(
                        Pos2::new(x, rect.top() + 12.0),
                        egui::Align2::CENTER_CENTER,
                        title,
                        egui::FontId::proportional(16.0),
                        theme.text,
                    );
                    painter.text(
                        Pos2::new(x, rect.top() + 32.0),
                        egui::Align2::CENTER_CENTER,
                        subtitle,
                        egui::FontId::monospace(11.0),
                        theme.text,
                    );
                }

                // Tunnel pipe
                painter.rect_stroke(
                    Rect::from_min_max(Pos2::new(bst_x, top + 5.0), Pos2::new(upf_x, rect.bottom() - 5.0)),
                    6.0,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(128, 128, 128, 80)),
                    egui::StrokeKind::Inside,
                );
            }

            let mut clicked = None;
            for (row, tunnel) in self.tunnels.iter().enumerate() {
                let y = top + 30.0 + row as f32 * ROW_HEIGHT;
                let (from_x, to_x) = if tunnel.uplink { (bst_x, upf_x) } else { (upf_x, bst_x) };
                if self.draw_arrow(ui, row, &rect, from_x, to_x, y, tunnel, &theme) {
                    clicked = Some(tunnel.last_index);
                }
            }
            if let Some(index) = clicked {
                ui.ctx()
                    .data_mut(|data| data.insert_temp(egui::Id::new(NAVIGATE_REQUEST_ID), index));
            }
        });
    }
}

impl EventSubscriber for GtpuTunnel {
    fn name(&self) -> &'static str {
        "GTPU Tunnel"
    }

    fn group(&self) -> PanelGroup {
        PanelGroup::Nas
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {}

    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        self.update(event, index);
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        self.current = self.update(event, index);
    }

    fn on_events_cleared(&mut self) {
        *self = Self::default();
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("GTPU Tunnel")
            .resizable(true)
            .default_width(520.0)
            .open(open)
            .show(ctx, |ui| self.draw(ui));
        Ok(())
    }
}

impl super::PanelView for GtpuTunnel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.draw(ui);
    }
}
