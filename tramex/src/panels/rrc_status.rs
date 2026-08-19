//! Panel to display the RRC status
use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::PanelView;
use crate::theme::{ArrowColors, ChannelColors, ThemeColors};
use egui::{Color32, TextFormat};
use tramex_tools::data::{AdditionalInfos, Trace};
use tramex_tools::errors::TramexError;
use tramex_tools::interface::layer::Layer;
use tramex_tools::interface::parse_config::Technology;
use tramex_tools::interface::types::Direction;

/// Create a label with a background color (using Color32 directly)
fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) -> egui::Response {
    use egui::text::LayoutJob;
    let mut job = LayoutJob::default();
    let theme = ThemeColors::get(ui);

    // Use theme-aware text color - dark text on colored backgrounds for readability
    let text_color = if show { ChannelColors::TEXT_ON_COLOR } else { theme.text };

    let background = if show { color } else { Color32::TRANSPARENT };

    job.append(
        label,
        0.0,
        TextFormat {
            color: text_color,
            background,
            ..Default::default()
        },
    );
    ui.label(job)
}

/// Make a label with hover effect
fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) {
    make_label(ui, label, show, color);
}

/// Arrow direction
#[derive(Debug)]
enum ArrowDirection {
    /// Up arrow
    Up,

    /// Down arrow
    Down,
}

/// Arrow color
#[derive(Debug, Clone)]
enum ArrowColor {
    /// Green arrow
    Green,

    /// Black arrow
    Black,
}

/// Create an arrow
fn make_arrow(ui: &mut egui::Ui, direction: ArrowDirection, color: ArrowColor, font_id: &egui::FontId) {
    // ↑↓
    // ⇑⇓
    // ⇡⇣ chosen
    // ⮉⮋
    // ⬆⬇
    // ⇧⇩
    let content = match direction {
        ArrowDirection::Down => "⇣",
        ArrowDirection::Up => "⇡",
    };
    let theme = ThemeColors::get(ui);
    let current_color = match color {
        ArrowColor::Green => ArrowColors::ACTIVE,
        ArrowColor::Black => theme.text,
    };

    ui.label(egui::RichText::new(content).color(current_color).font(font_id.clone()));
}

/// RRC connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RrcState {
    /// UE is in IDLE state
    #[default]
    Idle,
    /// UE is in INACTIVE state (NR only)
    Inactive,
    /// UE is in CONNECTED state
    Connected,
}

/// RRC connection state machine configuration for different technologies
struct RrcStateMachine {
    /// Connection request message (implies UE is in IDLE)
    connection_request_msg: &'static str,
    /// Message that triggers transition from IDLE to CONNECTED
    idle_to_connected_msg: &'static str,
    /// Message that triggers transition from CONNECTED to IDLE
    connected_to_idle_msg: &'static str,
    /// Message that triggers transition to INACTIVE (NR only)
    to_inactive_msg: Option<&'static str>,
    /// Message that triggers transition from INACTIVE to CONNECTED (NR only)
    inactive_to_connected_msg: Option<&'static str>,
}

impl RrcStateMachine {
    /// Get state machine configuration for LTE
    const fn lte() -> Self {
        Self {
            connection_request_msg: "RRC connection request",
            idle_to_connected_msg: "RRC connection setup complete",
            connected_to_idle_msg: "RRC connection release",
            to_inactive_msg: None,
            inactive_to_connected_msg: None,
        }
    }

    /// Get state machine configuration for NR (5G)
    const fn nr() -> Self {
        Self {
            connection_request_msg: "RRC setup request",
            idle_to_connected_msg: "RRC setup complete",
            connected_to_idle_msg: "RRC release",
            to_inactive_msg: Some("RRC suspendConfig"), // TODO: this message should be in the AdditionalInfos
            inactive_to_connected_msg: Some("RRC resume complete"),
        }
    }

    /// Get state machine for a given technology
    fn for_technology(tech: Technology) -> Self {
        match tech {
            Technology::LTE => Self::lte(),
            Technology::NR => Self::nr(),
            Technology::Unknown => Self::lte(), // Default to LTE for unknown
        }
    }

    /// Case-insensitive match between a canal message and one of the configured
    /// state-machine messages.
    ///
    /// Both `is_pertinent_message` (used to decide whether to show the direction arrow)
    /// and every transition check below funnel through this single predicate, so the two
    /// can never disagree about whether a given message "is" a particular configured
    /// message (previously one used loose case-insensitive substring matching with extra
    /// hardcoded patterns while the other used strict, case-sensitive equality).
    fn message_is(candidate: &str, target: &str) -> bool {
        candidate.eq_ignore_ascii_case(target)
    }

    /// Check if a message is pertinent (affects RRC state)
    fn is_pertinent_message(&self, canal_msg: &str) -> bool {
        let msg_lower = canal_msg.to_lowercase();

        Self::message_is(canal_msg, self.connection_request_msg)
            || Self::message_is(canal_msg, self.idle_to_connected_msg)
            || Self::message_is(canal_msg, self.connected_to_idle_msg)
            || self.to_inactive_msg.is_some_and(|m| Self::message_is(canal_msg, m))
            || self.inactive_to_connected_msg.is_some_and(|m| Self::message_is(canal_msg, m))
            || msg_lower.contains("rrc setup")
            || msg_lower.contains("rrc connection setup")
    }
}

/// Check whether a raw log line mentions a signalling or data radio bearer (SRBx / DRBx).
fn line_has_bearer_id(line: &str) -> bool {
    line.split_whitespace().any(|token| {
        let digits = token.strip_prefix("SRB").or_else(|| token.strip_prefix("DRB"));
        digits.is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
    })
}

/// Check whether a trace, by itself, implies the RRC connection must already be active.
///
/// This is used as a fallback for when we start observing a trace mid-session and
/// therefore never saw the initiating "RRC setup request"/"RRC connection request"
/// messages: some lower-layer traffic can only happen once the UE is already
/// RRC CONNECTED, so we can shortcut straight to that state instead of staying stuck in
/// IDLE forever. Returns a short reason (for logging) when the shortcut applies.
fn shortcut_active_reason(event: &Trace) -> Option<&'static str> {
    match &event.additional_infos {
        // Traffic on the dedicated control channel is, by construction, only possible
        // once the UE is RRC CONNECTED (LTE: "DCCH", NR: "DCCH-NR").
        AdditionalInfos::RRCInfos(infos) if infos.canal.contains("DCCH") => Some("DCCH traffic"),
        // RLC/PDCP layers carry no structured info (see `AdditionalInfos::None`), but a
        // signalling (SRBx) or data (DRBx) radio bearer only exists once the RRC
        // connection is set up, so seeing one on those layers is enough to infer it.
        AdditionalInfos::None if matches!(event.layer, Layer::RLC | Layer::PDCP) => event
            .text
            .as_ref()
            .and_then(|lines| lines.first())
            .filter(|line| line_has_bearer_id(line))
            .map(|_| "SRB/DRB traffic"),
        _ => None,
    }
}

/// State change record for history
#[derive(Debug, Clone)]
struct StateChange {
    /// Index
    index: usize,
    /// State
    state: RrcState,
    /// Canal
    canal: Option<String>,
    /// Canal message
    canal_msg: Option<String>,
    /// Direction
    direction: Option<Direction>,
}

/// Panel to display the RRC status
pub struct RRCStatusPanel {
    /// Canal - display field, only ever written by `refresh_display`
    canal: Option<String>,

    /// Canal message - display field, only ever written by `refresh_display`
    canal_msg: Option<String>,

    /// Direction - display field, only ever written by `refresh_display`
    direction: Option<Direction>,

    /// Current index
    current_index: usize,

    /// Font id for arrows
    arrow_font_id: egui::FontId,

    /// Font Id
    #[allow(dead_code)]
    label_font_id: egui::FontId,

    /// RRC state currently shown in the UI. This is a pure function of
    /// `(state_history, current_index)` and must only be written by `refresh_display`
    /// (via `navigate_to_index`) - never directly from `process_rrc_event`. Keeping a
    /// single writer means the display can never drift ahead of `current_index`, e.g.
    /// when a whole batch of events is added while the user is still looking at an
    /// earlier one.
    rrc_state: RrcState,

    /// Running state used internally while walking forward through newly added events to
    /// build `state_history` (`process_rrc_event`). This is intentionally separate from
    /// `rrc_state`: it always reflects "the state after the *last added* event", which is
    /// not necessarily the event the user is currently looking at.
    build_state: RrcState,

    /// Current technology
    technology: Technology,

    /// History of state changes (for bidirectional navigation).
    ///
    /// Append-only for the lifetime of a loaded file/connection, so memory grows with the
    /// number of RRC (and bearer-shortcut) messages seen. That's fine for typical log
    /// sizes; if very large captures ever become a concern, this would need bounding
    /// (e.g. periodic compaction/snapshotting) rather than keeping every entry forever.
    state_history: Vec<StateChange>,
}

impl Default for RRCStatusPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RRCStatusPanel {
    /// Create a new instance of the LinkPanel
    pub fn new() -> Self {
        Self {
            arrow_font_id: egui::FontId::monospace(50.0),
            label_font_id: egui::FontId::proportional(14.0),
            direction: None,
            canal: None,
            canal_msg: None,
            current_index: 0,
            rrc_state: RrcState::Idle,
            build_state: RrcState::Idle,
            technology: Technology::Unknown,
            state_history: Vec::new(),
        }
    }

    /// Process an incoming trace and advance the *build* state machine.
    ///
    /// This only ever touches `self.build_state`/`self.state_history`; it never writes to
    /// the display fields (`rrc_state`, `canal`, `canal_msg`, `direction`). See
    /// `refresh_display` for the single place that does.
    fn process_rrc_event(&mut self, event: &Trace, index: usize, technology: Technology) {
        // Shortcut: infer an already-active connection from lower-layer traffic, useful
        // when we start observing mid-session (see `shortcut_active_reason`).
        if self.build_state != RrcState::Connected
            && let Some(reason) = shortcut_active_reason(event)
        {
            log::info!(
                "RRC Status: idx={} inferred active connection via shortcut ({}) - forcing build state to Connected",
                index,
                reason
            );
            self.build_state = RrcState::Connected;
            self.state_history.push(StateChange {
                index,
                state: self.build_state,
                canal: None,
                canal_msg: None,
                direction: event.additional_infos.get_direction(),
            });
        }

        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            log::debug!(
                "RRC Status: processing event idx={} canal={} msg={:?} dir={:?} build_state={:?} tech={:?}",
                index,
                infos.canal,
                infos.canal_msg,
                infos.direction,
                self.build_state,
                technology
            );
            let previous_state = self.build_state;
            let state_machine = RrcStateMachine::for_technology(technology);

            // Calculate new state based on message
            self.update_build_state(infos.canal_msg.as_str(), &state_machine);

            if self.build_state != previous_state {
                log::info!(
                    "RRC Status: build state changed at idx={} from {:?} to {:?} (msg={:?})",
                    index,
                    previous_state,
                    self.build_state,
                    infos.canal_msg
                );
            } else {
                log::debug!(
                    "RRC Status: no build state change at idx={} for msg={:?} (state remains {:?})",
                    index,
                    infos.canal_msg,
                    self.build_state
                );
            }

            // Record state change in history
            self.state_history.push(StateChange {
                index,
                state: self.build_state,
                canal: Some(infos.canal.to_owned()),
                canal_msg: Some(infos.canal_msg.to_owned()),
                direction: Some(infos.direction.clone()),
            });
        } else {
            log::debug!("RRC Status: event idx={} is not an RRC message", index);
        }
    }

    /// Update the fields actually shown in the UI so they reflect the state at
    /// `target_index`.
    ///
    /// This is the *only* function that writes to `rrc_state`/`canal`/`canal_msg`/
    /// `direction`. It's called both when new events are appended (`on_event_added`) and
    /// when the user navigates (`on_event_focused`), so the display can never silently
    /// jump ahead of `current_index` regardless of which path triggered it.
    fn refresh_display(&mut self, target_index: usize, focused_event: Option<&Trace>) {
        self.navigate_to_index(target_index);

        // Only show the direction arrow if the focused event is itself a pertinent RRC
        // message (e.g. broadcast messages like SIB1/SIB2 shouldn't light up an arrow).
        self.direction = match focused_event.map(|e| &e.additional_infos) {
            Some(AdditionalInfos::RRCInfos(infos)) => {
                let state_machine = RrcStateMachine::for_technology(self.technology);
                let pertinent = state_machine.is_pertinent_message(&infos.canal_msg);
                log::debug!(
                    "RRC Status: focused msg={:?} pertinent={} direction={:?}",
                    infos.canal_msg,
                    pertinent,
                    infos.direction
                );
                pertinent.then(|| infos.direction.clone())
            }
            _ => None,
        };
    }

    /// Restore `rrc_state`/`canal`/`canal_msg`/`direction` from `state_history` for the
    /// most recent state change at or before `target_index`.
    fn navigate_to_index(&mut self, target_index: usize) {
        log::debug!(
            "RRC Status: navigate_to_index target={} history_len={}",
            target_index,
            self.state_history.len()
        );
        // `state_history` is append-only and strictly sorted by `index`, so a binary
        // search for the last entry at or before `target_index` is enough - no need to
        // scan backward through the whole history on every navigation.
        let pos = self.state_history.partition_point(|sc| sc.index <= target_index);
        if let Some(state_change) = pos.checked_sub(1).and_then(|i| self.state_history.get(i)) {
            log::debug!(
                "RRC Status: restoring state at idx={} -> {:?} (msg={:?})",
                state_change.index,
                state_change.state,
                state_change.canal_msg
            );
            self.rrc_state = state_change.state;
            self.canal = state_change.canal.clone();
            self.canal_msg = state_change.canal_msg.clone();
            self.direction = state_change.direction.clone();
        } else {
            log::debug!("RRC Status: no state found at or before target={}", target_index);
        }
    }

    /// Display the new UI layout
    fn ui_new_layout(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left column: Base station, arrows, user equipment
            ui.vertical(|ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new("BASE STATION").size(12.0).strong());
                ui.add_space(10.0);

                // Arrows
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    // Down arrow (DL: Base Station -> UE)
                    let dl_color = if matches!(self.direction, Some(Direction::DL)) {
                        ArrowColor::Green
                    } else {
                        ArrowColor::Black
                    };
                    make_arrow(ui, ArrowDirection::Down, dl_color, &self.arrow_font_id);

                    ui.add_space(15.0);

                    // Up arrow (UL: UE -> Base Station)
                    let ul_color = if matches!(self.direction, Some(Direction::UL)) {
                        ArrowColor::Green
                    } else {
                        ArrowColor::Black
                    };
                    make_arrow(ui, ArrowDirection::Up, ul_color, &self.arrow_font_id);
                });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("USER EQUIPMENT").size(12.0).strong());
            });

            ui.separator();

            // Right column: RRC states
            ui.vertical(|ui| {
                ui.add_space(5.0);

                // CONNECTED state
                let connected_active = self.rrc_state == RrcState::Connected;
                make_label_hover(ui, "CONNECTED", connected_active, ChannelColors::GREEN);

                ui.add_space(8.0);

                // IDLE state
                let idle_active = self.rrc_state == RrcState::Idle;
                make_label_hover(ui, "IDLE", idle_active, ChannelColors::RED);

                // INACTIVE state (only for NR)
                if self.technology == Technology::NR {
                    ui.add_space(8.0);
                    let inactive_active = self.rrc_state == RrcState::Inactive;
                    make_label_hover(ui, "INACTIVE", inactive_active, ChannelColors::GREEN);
                }
            });
        });
    }
}

// EventSubscriber implementation for new event system
impl EventSubscriber for RRCStatusPanel {
    fn name(&self) -> &'static str {
        "RRC Status"
    }

    fn on_metadata_changed(&mut self, metadata: &tramex_tools::interface::parse_config::FileMetadata) {
        // Update technology from context metadata
        if self.technology != metadata.technology {
            self.technology = metadata.technology;
            log::info!("RRC Status: Updated technology from context to {:?}", self.technology);
        }
    }
    fn on_event_added(&mut self, event: &Trace, index: usize, context: &EventContext) {
        self.process_rrc_event(event, index, self.technology);

        // Keep the displayed state in sync with whatever event is currently focused.
        // Without this, loading a batch of events would silently advance the display to
        // the state of the *last* event added, even though the user (and every other
        // panel) is still looking at `current_index`. Only bother refreshing while the
        // newly added event could actually affect what's shown at `current_index`
        // (i.e. it's at or before it); events added beyond it can't change the answer.
        if index <= self.current_index {
            let focused = context.all_events.get(self.current_index);
            self.refresh_display(self.current_index, focused);
        }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        log::debug!("RRC Status: event focused idx={}", index);
        // When user navigates, restore state at that point in time
        self.current_index = index;
        self.refresh_display(index, Some(event));
        log::debug!(
            "RRC Status: after refresh state={:?} canal={:?} msg={:?} dir={:?}",
            self.rrc_state,
            self.canal,
            self.canal_msg,
            self.direction
        );
    }

    fn on_events_cleared(&mut self) {
        log::debug!("RRC Status: Clearing all state history");
        self.canal = None;
        self.canal_msg = None;
        self.direction = None;
        self.current_index = 0;
        self.rrc_state = RrcState::Idle;
        self.build_state = RrcState::Idle;
        self.technology = Technology::Unknown;
        self.state_history.clear();
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("RRC Status")
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

impl RRCStatusPanel {
    /// Update `build_state` based on an RRC message.
    fn update_build_state(&mut self, canal_msg: &str, state_machine: &RrcStateMachine) {
        log::debug!(
            "RRC Status: update_build_state msg={:?} build_state={:?} tech={:?}",
            canal_msg,
            self.build_state,
            self.technology
        );
        // Connection request implies UE is in IDLE state
        if RrcStateMachine::message_is(canal_msg, state_machine.connection_request_msg) {
            log::debug!("RRC Status: matched connection request -> Idle");
            self.build_state = RrcState::Idle;
            return;
        }

        // Check for IDLE -> CONNECTED
        if self.build_state == RrcState::Idle
            && RrcStateMachine::message_is(canal_msg, state_machine.idle_to_connected_msg)
        {
            log::debug!("RRC Status: matched idle->connected transition");
            self.build_state = RrcState::Connected;
            return;
        }

        // Check for CONNECTED -> IDLE
        if self.build_state == RrcState::Connected
            && RrcStateMachine::message_is(canal_msg, state_machine.connected_to_idle_msg)
        {
            log::debug!("RRC Status: matched connected->idle transition");
            self.build_state = RrcState::Idle;
            return;
        }

        // NR-specific transitions
        if self.technology == Technology::NR {
            // CONNECTED -> INACTIVE (suspend)
            if let Some(suspend_msg) = state_machine.to_inactive_msg
                && self.build_state == RrcState::Connected
                && RrcStateMachine::message_is(canal_msg, suspend_msg)
            {
                log::debug!("RRC Status: matched connected->inactive transition");
                self.build_state = RrcState::Inactive;
                return;
            }

            // INACTIVE -> CONNECTED (resume)
            if let Some(resume_msg) = state_machine.inactive_to_connected_msg
                && self.build_state == RrcState::Inactive
                && RrcStateMachine::message_is(canal_msg, resume_msg)
            {
                log::debug!("RRC Status: matched inactive->connected transition");
                self.build_state = RrcState::Connected;
                return;
            }
        }

        log::debug!("RRC Status: no transition matched for msg={:?}", canal_msg);
    }
}

impl super::PanelView for RRCStatusPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_new_layout(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tramex_tools::interface::association::TraceRelation;
    use tramex_tools::interface::parser::parser_rrc::RRCInfos;

    fn trace(layer: Layer, additional_infos: AdditionalInfos, text: Option<Vec<String>>) -> Trace {
        Trace {
            timestamp: 0,
            layer,
            additional_infos,
            text,
            binary: None,
            relation: TraceRelation::default(),
        }
    }

    #[test]
    fn line_has_bearer_id_detects_srb_and_drb() {
        assert!(line_has_bearer_id("10:32:29.569 [RLC] UL 0001 SRB2 D/C=1 P=1 SI=00 SN=0"));
        assert!(line_has_bearer_id("10:32:29.600 [PDCP] DL 0001 DRB2 D/C=1 SN=0"));
        assert!(!line_has_bearer_id("10:32:29.569 [RLC] UL 0001 D/C=1 P=1 SI=00 SN=0"));
        // Not a valid bearer id (no digits after the prefix)
        assert!(!line_has_bearer_id("some SRB text without a number"));
    }

    #[test]
    fn message_is_case_insensitive() {
        assert!(RrcStateMachine::message_is("RRC setup complete", "RRC setup complete"));
        assert!(RrcStateMachine::message_is("rrc SETUP Complete", "RRC setup complete"));
        assert!(!RrcStateMachine::message_is("RRC setup", "RRC setup complete"));
    }

    #[test]
    fn bare_setup_message_is_pertinent_but_does_not_change_state() {
        let nr = RrcStateMachine::nr();
        // The network's bare "RRC setup" response is part of the handshake (pertinent,
        // shows a direction arrow) but is not itself a configured transition message.
        assert!(nr.is_pertinent_message("RRC setup"));
        assert!(!RrcStateMachine::message_is("RRC setup", nr.idle_to_connected_msg));
        assert!(!RrcStateMachine::message_is("RRC setup", nr.connection_request_msg));

        let lte = RrcStateMachine::lte();
        assert!(lte.is_pertinent_message("RRC connection setup"));
        assert!(!RrcStateMachine::message_is("RRC connection setup", lte.idle_to_connected_msg));
    }

    #[test]
    fn shortcut_detects_dcch_traffic() {
        let event = trace(
            Layer::RRC,
            AdditionalInfos::RRCInfos(RRCInfos {
                direction: Direction::DL,
                canal: "DCCH-NR".to_string(),
                canal_msg: "DL information transfer".to_string(),
            }),
            None,
        );
        assert_eq!(shortcut_active_reason(&event), Some("DCCH traffic"));
    }

    #[test]
    fn shortcut_ignores_common_control_channel() {
        let event = trace(
            Layer::RRC,
            AdditionalInfos::RRCInfos(RRCInfos {
                direction: Direction::UL,
                canal: "CCCH-NR".to_string(),
                canal_msg: "RRC setup request".to_string(),
            }),
            None,
        );
        assert_eq!(shortcut_active_reason(&event), None);
    }

    #[test]
    fn shortcut_detects_bearer_traffic_on_rlc_and_pdcp() {
        let rlc = trace(
            Layer::RLC,
            AdditionalInfos::None,
            Some(vec!["10:32:29.629 [RLC] UL 0001 DRB2 D/C=0 CPT=0 ACK_SN=1".to_string()]),
        );
        assert_eq!(shortcut_active_reason(&rlc), Some("SRB/DRB traffic"));

        let pdcp = trace(
            Layer::PDCP,
            AdditionalInfos::None,
            Some(vec!["10:32:29.570 [PDCP] DL 0001 SRB1 SN=7".to_string()]),
        );
        assert_eq!(shortcut_active_reason(&pdcp), Some("SRB/DRB traffic"));
    }

    #[test]
    fn shortcut_ignores_other_layers_and_missing_bearer_id() {
        let mac = trace(Layer::MAC, AdditionalInfos::None, Some(vec!["some MAC line".to_string()]));
        assert_eq!(shortcut_active_reason(&mac), None);

        let rlc_no_bearer = trace(
            Layer::RLC,
            AdditionalInfos::None,
            Some(vec!["10:32:29.629 [RLC] UL 0001 D/C=0 CPT=0 ACK_SN=1".to_string()]),
        );
        assert_eq!(shortcut_active_reason(&rlc_no_bearer), None);
    }
}
