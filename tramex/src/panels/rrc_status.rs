//! Panel to display the RRC status
use super::functions_panels::ArrowColor;
use super::functions_panels::ArrowDirection;
use super::functions_panels::make_arrow;
use super::functions_panels::make_label;
use crate::event_system::{EventContext, EventSubscriber};
use crate::panels::PanelView;
use crate::theme::ChannelColors;
use egui::Color32;
use tramex_tools::data::{AdditionalInfos, Trace};
use tramex_tools::errors::TramexError;
use tramex_tools::interface::parse_config::Technology;
use tramex_tools::interface::types::Direction;

/// Make a label with hover effect
fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: Color32) {
    make_label(ui, label, show, color);
}

/// RRC connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
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

    /// Check if a message is pertinent (affects RRC state)
    fn is_pertinent_message(&self, canal_msg: &str) -> bool {
        let msg_lower = canal_msg.to_lowercase();

        // Check all state-changing messages
        msg_lower.contains(&self.connection_request_msg.to_lowercase())
            || msg_lower.contains(&self.idle_to_connected_msg.to_lowercase())
            || msg_lower.contains(&self.connected_to_idle_msg.to_lowercase())
            || self
                .to_inactive_msg
                .map(|m| msg_lower.contains(&m.to_lowercase()))
                .unwrap_or(false)
            || self
                .inactive_to_connected_msg
                .map(|m| msg_lower.contains(&m.to_lowercase()))
                .unwrap_or(false)
            || msg_lower.contains("rrc setup")
            || msg_lower.contains("rrc connection setup")
    }
}

/// State change record for history
#[derive(Debug, Clone)]
struct StateChange {
    index: usize,
    state: RrcState,
    canal: Option<String>,
    canal_msg: Option<String>,
    direction: Option<Direction>,
}

/// Panel to display the RRC status
pub struct RRCStatusPanel {
    /// Canal
    canal: Option<String>,

    /// Canal message
    canal_msg: Option<String>,

    /// Direction
    direction: Option<Direction>,

    /// Current index
    current_index: usize,

    /// Font id for arrows
    arrow_font_id: egui::FontId,

    #[allow(dead_code)]
    label_font_id: egui::FontId,

    /// Current RRC state
    rrc_state: RrcState,

    /// Current technology
    technology: Technology,

    /// History of state changes (for bidirectional navigation)
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
            technology: Technology::Unknown,
            state_history: Vec::new(),
        }
    }

    /// Process an RRC event and update state
    fn process_rrc_event(&mut self, event: &Trace, index: usize, technology: Technology) {
        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            let state_machine = RrcStateMachine::for_technology(technology);

            // Calculate new state based on message
            self.update_connection_state_forward(infos.canal_msg.as_str(), &state_machine);

            // Record state change in history
            self.state_history.push(StateChange {
                index,
                state: self.rrc_state,
                canal: Some(infos.canal.to_owned()),
                canal_msg: Some(infos.canal_msg.to_owned()),
                direction: Some(infos.direction.clone()),
            });
        }
    }

    /// Navigate to a specific event by index
    fn navigate_to_index(&mut self, target_index: usize) {
        // Find the most recent state change at or before this index
        if let Some(state_change) = self.state_history.iter().filter(|sc| sc.index <= target_index).next_back() {
            self.rrc_state = state_change.state;
            self.canal = state_change.canal.clone();
            self.canal_msg = state_change.canal_msg.clone();
            self.direction = state_change.direction.clone();
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
    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        self.process_rrc_event(event, index, self.technology);
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // When user navigates, restore state at that point in time
        self.current_index = index;
        self.navigate_to_index(index);

        // Check if the focused event is a pertinent RRC message
        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            let state_machine = RrcStateMachine::for_technology(self.technology);

            // Only show direction arrow if this is a pertinent message (affects RRC state)
            if state_machine.is_pertinent_message(&infos.canal_msg) {
                self.direction = Some(infos.direction.clone());
            } else {
                // Non-pertinent RRC message (e.g., SIB1, SIB2) - clear direction
                self.direction = None;
            }
        } else {
            // Not an RRC message - clear direction
            self.direction = None;
        }
    }

    fn on_events_cleared(&mut self) {
        log::debug!("RRC Status: Clearing all state history");
        self.canal = None;
        self.canal_msg = None;
        self.direction = None;
        self.current_index = 0;
        self.rrc_state = RrcState::Idle;
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
    /// Update connection state based on RRC message
    fn update_connection_state_forward(&mut self, canal_msg: &str, state_machine: &RrcStateMachine) {
        // Connection request implies UE is in IDLE state
        if canal_msg == state_machine.connection_request_msg {
            self.rrc_state = RrcState::Idle;
            return;
        }

        // Check for IDLE -> CONNECTED
        if self.rrc_state == RrcState::Idle && canal_msg == state_machine.idle_to_connected_msg {
            self.rrc_state = RrcState::Connected;
            return;
        }

        // Check for CONNECTED -> IDLE
        if self.rrc_state == RrcState::Connected && canal_msg == state_machine.connected_to_idle_msg {
            self.rrc_state = RrcState::Idle;
            return;
        }

        // NR-specific transitions
        if self.technology == Technology::NR {
            // CONNECTED -> INACTIVE (suspend)
            if let Some(suspend_msg) = state_machine.to_inactive_msg
                && self.rrc_state == RrcState::Connected && canal_msg == suspend_msg {
                    self.rrc_state = RrcState::Inactive;
                    return;
                }

            // INACTIVE -> CONNECTED (resume)
            if let Some(resume_msg) = state_machine.inactive_to_connected_msg
                && self.rrc_state == RrcState::Inactive && canal_msg == resume_msg {
                    self.rrc_state = RrcState::Connected;
                }
        }
    }
}

impl super::PanelView for RRCStatusPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_new_layout(ui);
    }
}
