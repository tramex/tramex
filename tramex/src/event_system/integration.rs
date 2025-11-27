//! Integration helpers for migrating to the new event system

use super::Application;
use tramex_tools::data::Data;

/// Bridge between old Data struct and new Application
/// This allows gradual migration
pub struct EventSystemBridge {
    /// New event system application
    pub app: Application,
    
    /// Whether to use the new system (for A/B testing during migration)
    pub enabled: bool,
}

impl EventSystemBridge {
    /// Create a new bridge with the application
    pub fn new(app: Application) -> Self {
        Self {
            app,
            enabled: false, // Start disabled during migration
        }
    }
    
    /// Sync events from old Data struct to new EventStore
    /// Call this when old system loads new data
    pub fn sync_from_old_data(&mut self, old_data: &Data) {
        if !self.enabled {
            return;
        }
        
        let current_count = self.app.event_count();
        let new_count = old_data.events.len();
        
        // If old system has more events, sync them
        if new_count > current_count {
            let new_events: Vec<_> = old_data.events[current_count..].to_vec();
            
            // Process through the event system
            // Note: We'd need to expose process_new_events or add events directly
            // For now, this is a placeholder
            log::debug!("EventSystemBridge: Would sync {} new events", new_events.len());
        }
    }
    
    /// Sync current index from old Data to new Application
    pub fn sync_navigation(&mut self, old_data: &Data) {
        if !self.enabled {
            return;
        }
        
        if old_data.current_index != self.app.current_index() {
            self.app.navigate_to(old_data.current_index);
        }
    }
    
    /// Sync current index from new Application to old Data
    pub fn sync_navigation_reverse(&self, old_data: &mut Data) {
        if !self.enabled {
            return;
        }
        
        old_data.current_index = self.app.current_index();
    }
    
    /// Enable the new system
    pub fn enable(&mut self) {
        log::info!("EventSystemBridge: Enabling new event system");
        self.enabled = true;
    }
    
    /// Disable the new system (fall back to old)
    pub fn disable(&mut self) {
        log::info!("EventSystemBridge: Disabling new event system");
        self.enabled = false;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Helper to create an Application with all panels as subscribers
pub fn create_application_with_panels() -> Application {
    let mut app = Application::new();
    
    // Register all panels that have been migrated to EventSubscriber
    app.subscribe(Box::new(crate::panels::chronograph::Chronograph::new()));
    app.subscribe(Box::new(crate::panels::rrc_status::RRCStatusPanel::new()));
    app.subscribe(Box::new(crate::panels::bst_config::BstConfig::new()));
    app.subscribe(Box::new(crate::panels::logical_channels::LogicalChannels::new()));
    app.subscribe(Box::new(crate::panels::panel_message::MessageBox::new()));
    
    log::info!("Application created with {} subscribers", app.subscriber_count());
    app
}

/// Helper to check if we should use new system based on feature flag
pub fn should_use_new_system() -> bool {
    // Could be controlled by env var or config file during migration
    std::env::var("TRAMEX_USE_NEW_EVENT_SYSTEM")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bridge_creation() {
        let app = Application::new();
        let bridge = EventSystemBridge::new(app);
        
        assert!(!bridge.is_enabled());
    }
    
    #[test]
    fn test_bridge_enable_disable() {
        let app = Application::new();
        let mut bridge = EventSystemBridge::new(app);
        
        bridge.enable();
        assert!(bridge.is_enabled());
        
        bridge.disable();
        assert!(!bridge.is_enabled());
    }
}
