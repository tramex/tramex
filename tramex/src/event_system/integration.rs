//! Integration helpers for the event system

use super::Application;

/// Helper to create an Application with all panels as subscribers
pub fn create_application_with_panels() -> Application {
    let mut app = Application::new();

    // Register all panels
    app.subscribe(Box::new(crate::panels::chronograph::Chronograph::new()));
    app.subscribe(Box::new(crate::panels::rrc_status::RRCStatusPanel::new()));
    app.subscribe(Box::new(crate::panels::bst_config::BstConfig::new()));
    app.subscribe(Box::new(crate::panels::logical_channels::LogicalChannels::new()));
    app.subscribe(Box::new(crate::panels::panel_message::MessageBox::new()));
    app.subscribe(Box::new(crate::panels::identity::Identity::new()));
    app.subscribe(Box::new(crate::panels::resources_blocks::ResourceBlocks::new()));
    app.subscribe(Box::new(crate::panels::harq_panel::HarqPanel::new()));

    log::info!("Application created with {} subscribers", app.subscriber_count());
    app
}
