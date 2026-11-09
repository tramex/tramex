//! Panel to display the radio bearer status of the BST
//!
//! Shows the signaling (SRB) and traffic (DRB) radio bearer state machines.
//! The state only changes when an RRC message configuring (or releasing) a
//! bearer is passed, like the RRC Status panel.
use crate::event_system::{EventContext, EventSubscriber, PanelGroup};
use crate::panels::PanelView;
use crate::theme::{ArrowColors, ChannelColors, ThemeColors};
use egui::{Color32, TextFormat};
use tramex_tools::data::{AdditionalInfos, Trace};
use tramex_tools::errors::TramexError;
use tramex_tools::interface::types::Direction;

/// Signaling radio bearer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SrbState {
    /// No signaling bearer established
    #[default]
    None,
    /// SRB1 established
    Srb1,
    /// SRB2 established
    Srb2,
    /// SRB3 established
    Srb3,
}

/// Data radio bearer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DrbState {
    /// No data bearer established
    #[default]
    None,
    /// DRB1 established
    Drb1,
    /// DRB2 established
    Drb2,
    /// DRB3 established
    Drb3,
}

/// Target of a bearer transition
#[derive(Debug, Clone, Copy)]
enum BearerTarget {
    /// Signaling bearer
    Srb(SrbState),
    /// Data bearer
    Drb(DrbState),
}

/// A bearer transition: `message` carrying `identity_field identity` inside `list`
/// moves the matching state machine to `target`.
struct BearerTransition {
    /// RRC messages (canal message) triggering the transition
    messages: &'static [&'static str],
    /// ASN.1 list containing the bearer (e.g. "srb-ToAddModList")
    list: &'static str,
    /// Identity field inside the list (e.g. "srb-Identity")
    identity_field: &'static str,
    /// Bearer identity
    identity: u32,
    /// Resulting state
    target: BearerTarget,
}

/// Messages that start a new RRC connection: every bearer is reset before
/// applying the transitions of the message (e.g. RRC setup only adds SRB1).
const CONNECTION_SETUP_MSGS: &[&str] = &["RRC setup", "RRC connection setup"];

/// Messages that release every bearer
const CONNECTION_RELEASE_MSGS: &[&str] = &["RRC release", "RRC connection release"];

/// RRC reconfiguration messages (NR and LTE names)
const RECONFIGURATION_MSGS: &[&str] = &["RRC reconfiguration", "RRC connection reconfiguration"];

/// Bearer transitions, applied in order (the last matching one wins for each bearer type)
const TRANSITIONS: &[BearerTransition] = &[
    // RRC setup > radioBearerConfig > srb-ToAddModList > srb-Identity 1
    BearerTransition {
        messages: CONNECTION_SETUP_MSGS,
        list: "srb-ToAddModList",
        identity_field: "srb-Identity",
        identity: 1,
        target: BearerTarget::Srb(SrbState::Srb1),
    },
    // RRC reconfiguration > radioBearerConfig > srb-ToAddModList > srb-Identity 2
    BearerTransition {
        messages: RECONFIGURATION_MSGS,
        list: "srb-ToAddModList",
        identity_field: "srb-Identity",
        identity: 2,
        target: BearerTarget::Srb(SrbState::Srb2),
    },
    // TODO: SRB3 - add the message type and path, e.g.
    // BearerTransition { messages: &[...], list: "...", identity_field: "srb-Identity", identity: 3,
    //                    target: BearerTarget::Srb(SrbState::Srb3) },
    //
    // RRC reconfiguration > radioBearerConfig > drb-ToAddModList > drb-Identity 1
    BearerTransition {
        messages: RECONFIGURATION_MSGS,
        list: "drb-ToAddModList",
        identity_field: "drb-Identity",
        identity: 1,
        target: BearerTarget::Drb(DrbState::Drb1),
    },
    // RRC reconfiguration > radioBearerConfig > drb-ToAddModList > drb-Identity 2
    BearerTransition {
        messages: RECONFIGURATION_MSGS,
        list: "drb-ToAddModList",
        identity_field: "drb-Identity",
        identity: 2,
        target: BearerTarget::Drb(DrbState::Drb2),
    },
    // TODO: DRB3 - add the message type and path, e.g.
    // BearerTransition { messages: &[...], list: "drb-ToAddModList", identity_field: "drb-Identity", identity: 3,
    //                    target: BearerTarget::Drb(DrbState::Drb3) },
];

/// Case-insensitive match of a canal message against a list of messages
fn message_in(canal_msg: &str, messages: &[&str]) -> bool {
    messages.iter().any(|m| canal_msg.eq_ignore_ascii_case(m))
}

/// Collect the identities `identity_field N` found inside the ASN.1 block `list { ... }`.
///
/// Fields written as `field: N` (e.g. `servedRadioBearer srb-Identity: 1` in the
/// cell group config) are not bearer additions and are ignored.
fn identities_in_list(lines: &[String], list: &str, identity_field: &str) -> Vec<u32> {
    let mut ids = Vec::new();
    let mut depth: i32 = 0;
    for line in lines {
        let trimmed = line.trim();
        if depth == 0 {
            if trimmed.starts_with(list) {
                depth = brace_delta(trimmed);
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(identity_field)
            && rest.starts_with(char::is_whitespace)
            && let Ok(id) = rest.trim().trim_end_matches(',').parse()
        {
            ids.push(id);
        }
        depth += brace_delta(trimmed);
        depth = depth.max(0);
    }
    ids
}

/// Net number of opened braces in a line
fn brace_delta(line: &str) -> i32 {
    line.chars().fold(0, |acc, c| match c {
        '{' => acc + 1,
        '}' => acc - 1,
        _ => acc,
    })
}

/// Bearer states after `event`, starting from `(srb, drb)`.
/// Returns `None` if the event does not affect the bearers.
fn next_states(event: &Trace, srb: SrbState, drb: DrbState) -> Option<(SrbState, DrbState)> {
    let AdditionalInfos::RRCInfos(infos) = &event.additional_infos else {
        return None;
    };
    let msg = infos.canal_msg.as_str();
    if message_in(msg, CONNECTION_RELEASE_MSGS) {
        return Some((SrbState::None, DrbState::None));
    }
    let (mut srb, mut drb) = if message_in(msg, CONNECTION_SETUP_MSGS) {
        (SrbState::None, DrbState::None)
    } else {
        (srb, drb)
    };
    let mut matched = message_in(msg, CONNECTION_SETUP_MSGS);
    let lines = event.text.as_deref().unwrap_or_default();
    for transition in TRANSITIONS.iter().filter(|t| message_in(msg, t.messages)) {
        if identities_in_list(lines, transition.list, transition.identity_field).contains(&transition.identity) {
            matched = true;
            match transition.target {
                BearerTarget::Srb(s) => srb = s,
                BearerTarget::Drb(d) => drb = d,
            }
        }
    }
    matched.then_some((srb, drb))
}

/// State change record for history
#[derive(Debug, Clone)]
struct StateChange {
    /// Trace index
    index: usize,
    /// SRB state after this trace
    srb: SrbState,
    /// DRB state after this trace
    drb: DrbState,
}

/// Create a label with a background color
fn make_label(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) {
    let mut job = egui::text::LayoutJob::default();
    let theme = ThemeColors::get(ui);
    job.append(
        label,
        0.0,
        TextFormat {
            color: if show { ChannelColors::TEXT_ON_COLOR } else { theme.text },
            background: if show { color } else { Color32::TRANSPARENT },
            ..Default::default()
        },
    );
    ui.label(job);
}

/// Create an arrow, green when active
fn make_arrow(ui: &mut egui::Ui, content: &str, active: bool, font_id: &egui::FontId) {
    let color = if active {
        ArrowColors::ACTIVE
    } else {
        ThemeColors::get(ui).text
    };
    ui.label(egui::RichText::new(content).color(color).font(font_id.clone()));
}

/// Panel to display the SRB / DRB status
pub struct BearerStatusPanel {
    /// SRB state shown in the UI - only written by `refresh_display`
    srb: SrbState,
    /// DRB state shown in the UI - only written by `refresh_display`
    drb: DrbState,
    /// Direction of the focused message, if it changes the bearers
    direction: Option<Direction>,
    /// Running SRB state after the last added event (used to build `state_history`)
    build_srb: SrbState,
    /// Running DRB state after the last added event (used to build `state_history`)
    build_drb: DrbState,
    /// Current focused index
    current_index: usize,
    /// Font id for arrows
    arrow_font_id: egui::FontId,
    /// History of state changes, sorted by index (for bidirectional navigation)
    state_history: Vec<StateChange>,
}

impl Default for BearerStatusPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl BearerStatusPanel {
    /// Create a new Bearer Status panel
    pub fn new() -> Self {
        Self {
            srb: SrbState::None,
            drb: DrbState::None,
            direction: None,
            build_srb: SrbState::None,
            build_drb: DrbState::None,
            current_index: 0,
            arrow_font_id: egui::FontId::monospace(50.0),
            state_history: Vec::new(),
        }
    }

    /// Advance the build state machine with a newly added event
    fn process_event(&mut self, event: &Trace, index: usize) {
        if self.state_history.last().is_some_and(|sc| sc.index >= index) {
            return;
        }
        if let Some((srb, drb)) = next_states(event, self.build_srb, self.build_drb) {
            log::debug!("Bearer Status: idx={} srb={:?} drb={:?}", index, srb, drb);
            self.build_srb = srb;
            self.build_drb = drb;
            self.state_history.push(StateChange { index, srb, drb });
        }
    }

    /// Update the displayed state to reflect `target_index`
    fn refresh_display(&mut self, target_index: usize, focused_event: Option<&Trace>) {
        let pos = self.state_history.partition_point(|sc| sc.index <= target_index);
        let (srb, drb) = pos
            .checked_sub(1)
            .and_then(|i| self.state_history.get(i))
            .map_or((SrbState::None, DrbState::None), |sc| (sc.srb, sc.drb));
        self.srb = srb;
        self.drb = drb;

        // Light up the arrow only if the focused message itself changes the bearers
        self.direction = focused_event.and_then(|e| match &e.additional_infos {
            AdditionalInfos::RRCInfos(infos) if self.state_history.iter().any(|sc| sc.index == target_index) => {
                Some(infos.direction.clone())
            }
            _ => None,
        });
    }

    /// Display the panel
    fn ui_layout(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left column: Base station, arrows, user equipment
            ui.vertical(|ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new("BASE STATION").size(12.0).strong());
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    make_arrow(ui, "⇣", matches!(self.direction, Some(Direction::DL)), &self.arrow_font_id);
                    ui.add_space(15.0);
                    make_arrow(ui, "⇡", matches!(self.direction, Some(Direction::UL)), &self.arrow_font_id);
                });
                ui.add_space(10.0);
                ui.label(egui::RichText::new("USER EQUIPMENT").size(12.0).strong());
            });

            ui.separator();

            // SRB column (signaling)
            ui.vertical(|ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new("SRB (signaling)").size(12.0).strong());
                for (label, state) in [("SRB3", SrbState::Srb3), ("SRB2", SrbState::Srb2), ("SRB1", SrbState::Srb1)] {
                    ui.add_space(8.0);
                    make_label(ui, label, self.srb == state, ChannelColors::ORANGE);
                }
            });

            ui.separator();

            // DRB column (traffic)
            ui.vertical(|ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new("DRB (traffic)").size(12.0).strong());
                for (label, state) in [("DRB3", DrbState::Drb3), ("DRB2", DrbState::Drb2), ("DRB1", DrbState::Drb1)] {
                    ui.add_space(8.0);
                    make_label(ui, label, self.drb == state, ChannelColors::GREEN);
                }
            });
        });
    }
}

impl EventSubscriber for BearerStatusPanel {
    fn name(&self) -> &'static str {
        "Bearer Status"
    }

    fn group(&self) -> PanelGroup {
        PanelGroup::Rrc
    }

    fn on_metadata_changed(&mut self, _metadata: &tramex_tools::interface::parse_config::FileMetadata) {}

    fn on_event_added(&mut self, event: &Trace, index: usize, context: &EventContext) {
        self.process_event(event, index);
        if index <= self.current_index {
            let focused = context.all_events.get(self.current_index);
            self.refresh_display(self.current_index, focused);
        }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        self.current_index = index;
        self.refresh_display(index, Some(event));
    }

    fn on_events_cleared(&mut self) {
        *self = Self::new();
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Bearer Status")
            .resizable(true)
            .default_width(500.0)
            .open(open)
            .show(ctx, |ui| self.ui(ui));
        Ok(())
    }
}

impl PanelView for BearerStatusPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_layout(ui);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]
    use super::*;
    use tramex_tools::interface::association::TraceRelation;
    use tramex_tools::interface::layer::Layer;
    use tramex_tools::interface::parser::parser_rrc::RRCInfos;

    fn rrc_trace(canal_msg: &str, body: &str) -> Trace {
        Trace {
            timestamp: 0,
            layer: Layer::RRC,
            additional_infos: AdditionalInfos::RRCInfos(RRCInfos {
                direction: Direction::DL,
                canal: "DCCH-NR".to_string(),
                canal_msg: canal_msg.to_string(),
            }),
            text: Some(body.lines().map(str::to_string).collect()),
            binary: None,
            relation: TraceRelation::default(),
        }
    }

    const SETUP: &str = "13:21:07.039 [RRC] DL 0048 01 CCCH-NR: RRC setup
        {
          message c1: rrcSetup: {
            criticalExtensions rrcSetup: {
              radioBearerConfig {
                srb-ToAddModList {
                  {
                    srb-Identity 1
                  }
                }
              },
              masterCellGroup {
                rlc-BearerToAddModList {
                  {
                    servedRadioBearer srb-Identity: 1,";

    const RECONF: &str = "13:21:07.094 [RRC] DL 0048 01 DCCH-NR: RRC reconfiguration
        {
              radioBearerConfig {
                srb-ToAddModList {
                  {
                    srb-Identity 2
                  }
                },
                drb-ToAddModList {
                  {
                    cnAssociation sdap-Config: {
                      pdu-Session 5,
                      mappedQoS-FlowsToAdd {
                        1
                      }
                    },
                    drb-Identity 1,
                    pdcp-Config {
                    }
                  }
                }
              },
              servedRadioBearer drb-Identity: 2,";

    #[test]
    fn identities_ignore_other_blocks() {
        let lines: Vec<String> = RECONF.lines().map(str::to_string).collect();
        assert_eq!(identities_in_list(&lines, "srb-ToAddModList", "srb-Identity"), vec![2]);
        assert_eq!(identities_in_list(&lines, "drb-ToAddModList", "drb-Identity"), vec![1]);
    }

    #[test]
    fn state_machine_setup_reconf_release() {
        let setup = next_states(&rrc_trace("RRC setup", SETUP), SrbState::None, DrbState::Drb1);
        assert_eq!(setup, Some((SrbState::Srb1, DrbState::None)));

        let reconf = next_states(&rrc_trace("RRC reconfiguration", RECONF), SrbState::Srb1, DrbState::None);
        assert_eq!(reconf, Some((SrbState::Srb2, DrbState::Drb1)));

        let release = next_states(&rrc_trace("RRC release", ""), SrbState::Srb2, DrbState::Drb1);
        assert_eq!(release, Some((SrbState::None, DrbState::None)));
    }

    #[test]
    fn unrelated_message_keeps_state() {
        assert_eq!(
            next_states(&rrc_trace("DL information transfer", ""), SrbState::Srb2, DrbState::Drb1),
            None
        );
        // Reconfiguration without bearer config does not change anything
        assert_eq!(
            next_states(&rrc_trace("RRC reconfiguration", "{ }"), SrbState::Srb2, DrbState::Drb1),
            None
        );
    }

    #[test]
    fn navigation_restores_history() {
        let mut panel = BearerStatusPanel::new();
        panel.process_event(&rrc_trace("RRC setup", SETUP), 2);
        panel.process_event(&rrc_trace("RRC reconfiguration", RECONF), 5);
        panel.refresh_display(1, None);
        assert_eq!((panel.srb, panel.drb), (SrbState::None, DrbState::None));
        panel.refresh_display(3, None);
        assert_eq!((panel.srb, panel.drb), (SrbState::Srb1, DrbState::None));
        panel.refresh_display(9, None);
        assert_eq!((panel.srb, panel.drb), (SrbState::Srb2, DrbState::Drb1));
    }
}
