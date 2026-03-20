//! Application controller - orchestrates the event system

use super::{
    EventBus, EventStore, DataSource, EventContext, EventSubscriber,
};
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;
use tramex_tools::interface::association::AssociationRules;
use tramex_tools::errors::{TramexError, ErrorCode};

/// Main application controller that ties together the event system
pub struct Application {
    /// Event store - owns all events
    event_store: EventStore,
    
    /// Event bus - coordinates notifications
    event_bus: EventBus,
    
    /// Data source (File, WebSocket, etc.)
    data_source: Option<Box<dyn DataSource>>,
    
    /// Layer filters
    layers: Layers,
    
    /// File metadata (technology, version, etc.)
    metadata: FileMetadata,
    
    /// Association rules for computing trace relationships
    association_rules: AssociationRules,
}

impl Application {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            event_store: EventStore::new(),
            event_bus: EventBus::new(),
            data_source: None,
            layers: Layers::new_optiniated(),
            metadata: FileMetadata::default(),
            association_rules: AssociationRules::new(),
        }
    }
    
    /// Subscribe a panel to receive event notifications
    pub fn subscribe(&mut self, subscriber: Box<dyn EventSubscriber>) {
        log::debug!("Application: Subscribing panel '{}'", subscriber.name());
        self.event_bus.subscribe(subscriber);
    }
    
    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.event_bus.subscriber_count()
    }
    
    /// Set the data source
    pub fn set_data_source(&mut self, source: Box<dyn DataSource>) {
        log::info!("Application: Setting data source: {:?}", source.source_type());
        
        // Clear existing data when switching sources
        self.clear_all();
        
        self.data_source = Some(source);
    }
    
    /// Get the current data source type
    pub fn data_source_type(&self) -> Option<String> {
        self.data_source.as_ref().map(|s| {
            match s.source_type() {
                super::DataSourceType::File { path, .. } => {
                    format!("File: {}", path.display())
                }
                super::DataSourceType::WebSocket { url, .. } => {
                    format!("WebSocket: {}", url)
                }
            }
        })
    }
    
    /// Get layer filters
    pub fn layers(&self) -> &Layers {
        &self.layers
    }
    
    /// Get mutable layer filters
    pub fn layers_mut(&mut self) -> &mut Layers {
        &mut self.layers
    }
    
    /// Set layer filters
    pub fn set_layers(&mut self, layers: Layers) {
        self.layers = layers;
    }
    
    /// Update - poll data source and process new events
    /// Should be called every frame
    pub fn update(&mut self) -> Result<(), Vec<TramexError>> {
        let mut errors = Vec::new();
        
        // 1. Poll for new events (separate scope to drop borrow)
        let new_events = if let Some(source) = &mut self.data_source {
            match source.poll() {
                Ok(events) => events,
                Err(e) => {
                    errors.extend(e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        // 2. Process new events (source borrow dropped)
        if !new_events.is_empty() {
            log::debug!("Application: Polled {} new events", new_events.len());
            if let Err(e) = self.process_new_events(new_events) {
                errors.extend(e);
            }
        }
        
        // 3. Sync metadata from data source if changed
        let new_meta = self.data_source.as_ref()
            .and_then(|s| s.metadata())
            .filter(|m| m.technology != self.metadata.technology)
            .cloned();
        if let Some(meta) = new_meta {
            log::info!("Application: Synced metadata from source - Technology: {:?}", meta.technology);
            self.metadata = meta;
            self.event_bus.notify_metadata_changed(&self.metadata);
        }
        
        // 4. For auto-loading sources, request more data
        if let Some(source) = &mut self.data_source {
            if source.is_auto_loading() && source.has_more() {
                if let Err(e) = source.request_more(&self.layers) {
                    for error in e {
                        if !matches!(error.get_code(), ErrorCode::ParsingLayerNotImplemented) {
                            errors.push(error);
                        }
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Process a batch of new events
    fn process_new_events(&mut self, new_events: Vec<tramex_tools::data::Trace>) -> Result<(), Vec<TramexError>> {
        if new_events.is_empty() {
            return Ok(());
        }
        
        log::debug!("Application: Processing {} new events", new_events.len());
        
        // Add events to store
        let range = self.event_store.add_events(new_events);
        
        // Compute associations for the new batch (with lookback for cross-batch relationships)
        let batch_start = range.start;
        self.event_store.compute_associations(&self.association_rules, batch_start);
        
        // Create context for subscribers
        let context = EventContext {
            all_events: self.event_store.events(),
            current_index: self.event_store.current_index(),
            layers: &self.layers,
            metadata: &self.metadata,
        };
        
        // Notify all subscribers of new events
        for index in range {
            if let Some(event) = self.event_store.events().get(index) {
                self.event_bus.notify_event_added(event, index, &context);
            }
        }
        
        // For auto-loading sources, automatically navigate to the last event
        if let Some(source) = &self.data_source {
            if source.is_auto_loading() {
                if let Some(last_index) = self.event_store.len().checked_sub(1) {
                    self.navigate_to(last_index);
                }
            }
        }
        
        Ok(())
    }
    
    /// Navigate to next event (skips disabled layers)
    pub fn navigate_next(&mut self) -> bool {
        let start_index = self.event_store.current_index();
        let total_events = self.event_store.len();
        
        log::debug!("navigate_next: starting from index {}, total events loaded: {}", start_index, total_events);
        
        // Search for next enabled event
        loop {
            let can_continue = self.event_store.can_go_next();
            // log::debug!("can_go_next: {} (current: {}, total: {})", can_continue, self.event_store.current_index(), self.event_store.len());
            
            if !can_continue {
                log::debug!("Application: Reached end of events at index {}", self.event_store.current_index());
                // Restore original position since we didn't find an enabled event
                self.event_store.set_current(start_index);
                return false;
            }
            
            self.event_store.go_next();
            let index = self.event_store.current_index();
            
            if let Some(event) = self.event_store.current() {
                // Check if this event's layer is enabled
                if self.layers.is_layer_enabled(&event.layer) {
                    log::debug!("Application: Navigate next to index {} (layer: {:?})", index, event.layer);
                    
                    let context = EventContext {
                        all_events: self.event_store.events(),
                        current_index: index,
                        layers: &self.layers,
                        metadata: &self.metadata,
                    };
                    
                    self.event_bus.notify_event_focused(event, index, &context);
                    return true;
                }
                // Continue to next event if layer is disabled
                log::trace!("Skipping event at index {} (layer: {:?} is disabled)", index, event.layer);
            } else {
                // Shouldn't happen, but restore position and return false
                self.event_store.set_current(start_index);
                return false;
            }
        }
    }
    
    /// Navigate to previous event (skips disabled layers)
    pub fn navigate_previous(&mut self) -> bool {
        let start_index = self.event_store.current_index();
        
        // Search for previous enabled event
        loop {
            if !self.event_store.can_go_previous() {
                log::debug!("Application: Reached beginning of events");
                return false;
            }
            
            self.event_store.go_previous();
            let index = self.event_store.current_index();
            
            if let Some(event) = self.event_store.current() {
                // Check if this event's layer is enabled
                if self.layers.is_layer_enabled(&event.layer) {
                    log::debug!("Application: Navigate previous to index {} (layer: {:?})", index, event.layer);
                    
                    let context = EventContext {
                        all_events: self.event_store.events(),
                        current_index: index,
                        layers: &self.layers,
                        metadata: &self.metadata,
                    };
                    
                    self.event_bus.notify_event_focused(event, index, &context);
                    return true;
                }
                // Continue to previous event if layer is disabled
            } else {
                // Shouldn't happen, but restore position and return false
                self.event_store.set_current(start_index);
                return false;
            }
        }
    }
    
    /// Navigate to specific index
    pub fn navigate_to(&mut self, index: usize) -> bool {
        if self.event_store.set_current(index).is_some() {
            log::debug!("Application: Navigate to index {}", index);
            
            // Get immutable references after set_current
            let event = self.event_store.current().unwrap(); // Safe: we just checked
            let context = EventContext {
                all_events: self.event_store.events(),
                current_index: index,
                layers: &self.layers,
                metadata: &self.metadata,
            };
            
            self.event_bus.notify_event_focused(event, index, &context);
            true
        } else {
            false
        }
    }
    
    /// Check if can navigate to next
    pub fn can_navigate_next(&self) -> bool {
        self.event_store.can_go_next()
    }
    
    /// Check if can navigate to previous
    pub fn can_navigate_previous(&self) -> bool {
        self.event_store.can_go_previous()
    }
    
    /// Get current event index
    pub fn current_index(&self) -> usize {
        self.event_store.current_index()
    }
    
    /// Get total number of loaded events
    pub fn event_count(&self) -> usize {
        self.event_store.len()
    }
    
    /// Get total event count if known
    pub fn total_event_count(&self) -> Option<usize> {
        self.data_source.as_ref()
            .and_then(|s| s.total_count())
            .or_else(|| self.event_store.total_count())
    }
    
    /// Check if data source is fully loaded
    pub fn is_fully_loaded(&self) -> bool {
        self.data_source.as_ref()
            .map(|s| !s.has_more())
            .unwrap_or(true)
    }
    
    /// Get loading progress
    pub fn progress(&self) -> Option<f32> {
        self.data_source.as_ref()
            .and_then(|s| s.progress())
            .or_else(|| self.event_store.progress())
    }
    
    /// Toggle auto-loading for current data source
    pub fn toggle_auto_loading(&mut self) {
        if let Some(source) = &mut self.data_source {
            source.toggle_auto_loading();
            log::info!("Application: Auto-loading toggled to {}", source.is_auto_loading());
        }
    }
    
    /// Check if auto-loading is enabled
    pub fn is_auto_loading(&self) -> bool {
        self.data_source.as_ref()
            .map(|s| s.is_auto_loading())
            .unwrap_or(false)
    }
    
    /// Check if data source has more data available
    pub fn has_more_data(&self) -> bool {
        self.data_source.as_ref()
            .map(|s| s.has_more())
            .unwrap_or(false)
    }
    
    /// Request more data from source (for on-demand loading)
    pub fn request_more_data(&mut self) -> Result<(), Vec<TramexError>> {
        if let Some(source) = &mut self.data_source {
            log::debug!("Application: Requesting more data from source");
            if let Err(e) = source.request_more(&self.layers) {
                log::error!("Application: request_more failed with {} errors", e.len());
                for err in &e {
                    log::error!("  - {:?}: {}", err.get_code(), err.message);
                }
                return Err(e);
            }
            log::debug!("Application: request_more succeeded");
        } else {
            log::warn!("Application: request_more_data called but no data source");
        }
        Ok(())
    }
    
    /// Clear all events and reset state
    pub fn clear_all(&mut self) {
        log::info!("Application: Clearing all data");
        
        self.event_store.clear();
        self.event_bus.notify_cleared();
    }
    
    /// Get reference to event store (for direct access if needed)
    pub fn event_store(&self) -> &EventStore {
        &self.event_store
    }
    
    /// Get current event
    pub fn current_event(&self) -> Option<&tramex_tools::data::Trace> {
        self.event_store.current()
    }
    
    /// Get all events
    pub fn events(&self) -> &[tramex_tools::data::Trace] {
        self.event_store.events()
    }
    
    /// Check if any data is loaded
    pub fn has_data(&self) -> bool {
        !self.event_store.is_empty()
    }
    
    /// Check if data source is available
    pub fn has_data_source(&self) -> bool {
        self.data_source.is_some()
    }
    
    /// Get reference to data source (for downcasting to concrete types)
    pub fn data_source(&self) -> Option<&dyn DataSource> {
        self.data_source.as_deref()
    }
    
    /// Get names of all subscribed panels
    pub fn panel_names(&self) -> Vec<&'static str> {
        self.event_bus.subscriber_names()
    }
    
    /// Show windows for all subscribed panels
    /// Returns (panel_name, is_open, result) so the caller can update open_windows
    pub fn show_panel_windows(
        &mut self,
        ctx: &egui::Context,
        open_windows: &std::collections::BTreeSet<String>,
    ) -> Vec<(String, bool, Result<(), TramexError>)> {
        self.event_bus.show_windows(ctx, open_windows)
    }
    
    /// Get file metadata
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }
    
    /// Set file metadata (should be called when loading a new file)
    pub fn set_metadata(&mut self, metadata: FileMetadata) {
        log::info!("Application: Setting metadata - Technology: {:?}", metadata.technology);
        self.metadata = metadata;
        self.event_bus.notify_metadata_changed(&self.metadata);
    }
    
    #[cfg(feature = "ai")]
    /// Forward AI config to all subscriber panels
    pub fn set_ai_config(&mut self, key: &str, provider: &tramex_tools::ai::AIProvider) {
        self.event_bus.set_ai_config(key, provider);
    }


}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
