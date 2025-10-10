//! Panel to display the RRC status
use super::functions_panels::ArrowColor;
use super::functions_panels::ArrowDirection;
use super::functions_panels::CustomLabelColor;
use super::functions_panels::make_arrow;
use super::functions_panels::make_label;
use tramex_tools::data::AdditionalInfos;
use tramex_tools::data::Data;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::types::Direction;
use tramex_tools::interface::parse_config::Technology;

/// Make a label with hover effect
fn make_label_hover(ui: &mut egui::Ui, label: &str, show: bool, color: CustomLabelColor) {
    make_label(ui, label, show, color);
}

/// RRC connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RrcState {
    /// UE is in IDLE state
    Idle,
    /// UE is in INACTIVE state (NR only)
    Inactive,
    /// UE is in CONNECTED state
    Connected,
}

impl Default for RrcState {
    fn default() -> Self {
        RrcState::Idle
    }
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
            to_inactive_msg: Some("RRC suspendConfig"), // TODO: this message sould be in the AdditionnalInfos
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

    /// Font id for labels
    label_font_id: egui::FontId,

    /// Current RRC state
    rrc_state: RrcState,

    /// Current technology
    technology: Technology,
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
                make_label_hover(ui, "CONNECTED", connected_active, CustomLabelColor::Green);
                
                ui.add_space(8.0);
                
                // IDLE state
                let idle_active = self.rrc_state == RrcState::Idle;
                make_label_hover(ui, "IDLE", idle_active, CustomLabelColor::Red);
                
                // INACTIVE state (only for NR)
                if self.technology == Technology::NR {
                    ui.add_space(8.0);
                    let inactive_active = self.rrc_state == RrcState::Inactive;
                    make_label_hover(ui, "INACTIVE", inactive_active, CustomLabelColor::Green);
                }
            });
        });
    }
}

impl super::PanelController for RRCStatusPanel {
    fn name(&self) -> &'static str {
        "RRC Status"
    }

    fn window_title(&self) -> &'static str {
        "RRC Status"
    }

    fn clear(&mut self) {
        self.canal = None;
        self.canal_msg = None;
        self.direction = None;
        self.current_index = 0;
        self.rrc_state = RrcState::Idle;
        self.technology = Technology::Unknown;
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Data) -> Result<(), TramexError> {
        // Update technology from data metadata
        if self.technology != data.metadata.technology {
            self.technology = data.metadata.technology;
        }

        if data.is_different_index(self.current_index) {
            if let Some(one_trace) = data.get_current_trace() {
                match &one_trace.additional_infos {
                    AdditionalInfos::RRCInfos(infos) => {
                        // Get state machine for current technology
                        let state_machine = RrcStateMachine::for_technology(self.technology);
                        
                        // Update RRC connection state based on direction (forward/backward)
                        if self.current_index < data.current_index {
                            // Moving forward in time
                            self.update_connection_state_forward(
                                infos.canal_msg.as_str(),
                                &state_machine
                            );
                        } else {
                            // Moving backward in time (reverse state transitions)
                            self.update_connection_state_backward(
                                infos.canal_msg.as_str(),
                                &state_machine
                            );
                        }

                        self.canal = Some(infos.canal.to_owned());
                        self.canal_msg = Some(infos.canal_msg.to_owned());
                        self.direction = Some(infos.direction.clone());
                    },
                    _ => {}
                }
            }

            self.current_index = data.current_index;
        }
        egui::Window::new(self.window_title())
            .default_width(160.0)
            .default_height(160.0)
            .open(open)
            .resizable([true, true])
            .show(ctx, |ui| {
                use super::PanelView as _;
                self.ui(ui)
            });
        Ok(())
    }
}

impl RRCStatusPanel {
    /// Update connection state when moving forward in time
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
            if let Some(suspend_msg) = state_machine.to_inactive_msg {
                if self.rrc_state == RrcState::Connected && canal_msg == suspend_msg {
                    self.rrc_state = RrcState::Inactive;
                    return;
                }
            }
            
            // INACTIVE -> CONNECTED (resume)
            if let Some(resume_msg) = state_machine.inactive_to_connected_msg {
                if self.rrc_state == RrcState::Inactive && canal_msg == resume_msg {
                    self.rrc_state = RrcState::Connected;
                    return;
                }
            }
        }
    }

    /// Update connection state when moving backward in time (reverse transitions)
    fn update_connection_state_backward(&mut self, canal_msg: &str, state_machine: &RrcStateMachine) {
        // Reverse: when going back and seeing a connection request, we were CONNECTED before it
        if canal_msg == state_machine.connection_request_msg {
            self.rrc_state = RrcState::Connected;
            return;
        }
        
        // Reverse: CONNECTED -> IDLE (when seeing setup complete)
        if self.rrc_state == RrcState::Connected && canal_msg == state_machine.idle_to_connected_msg {
            self.rrc_state = RrcState::Idle;
            return;
        }
        
        // Reverse: IDLE -> CONNECTED (when seeing release)
        if self.rrc_state == RrcState::Idle && canal_msg == state_machine.connected_to_idle_msg {
            self.rrc_state = RrcState::Connected;
            return;
        }
        
        // NR-specific reverse transitions
        if self.technology == Technology::NR {
            // Reverse: INACTIVE -> CONNECTED (when seeing suspend)
            if let Some(suspend_msg) = state_machine.to_inactive_msg {
                if self.rrc_state == RrcState::Inactive && canal_msg == suspend_msg {
                    self.rrc_state = RrcState::Connected;
                    return;
                }
            }
            
            // Reverse: CONNECTED -> INACTIVE (when seeing resume)
            if let Some(resume_msg) = state_machine.inactive_to_connected_msg {
                if self.rrc_state == RrcState::Connected && canal_msg == resume_msg {
                    self.rrc_state = RrcState::Inactive;
                    return;
                }
            }
        }
    }
}

impl super::PanelView for RRCStatusPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_new_layout(ui);
    }
}
