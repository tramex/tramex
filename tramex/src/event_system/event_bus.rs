//! Event bus for coordinating notifications to subscribers

use tramex_tools::data::Trace;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;
use egui;

/// Context provided to subscribers when events occur
pub struct EventContext<'a> {
    /// All events currently in the store
    pub all_events: &'a [Trace],
    /// Current focused event index
    pub current_index: usize,
    /// Active layer filters
    pub layers: &'a Layers,
    /// File metadata (technology, version, etc.)
    pub metadata: &'a FileMetadata,
}

/// Trait for components that want to be notified of events
pub trait EventSubscriber: Send {
    /// Called when a new event is added to the store
    /// This is where panels should extract and process event data
    fn on_event_added(&mut self, event: &Trace, index: usize, context: &EventContext);
    
    /// Called when user navigates to focus on a specific event
    /// This is for UI updates like scrolling, highlighting, etc.
    fn on_event_focused(&mut self, event: &Trace, index: usize, context: &EventContext);
    
    /// Called when all events are cleared
    fn on_events_cleared(&mut self);
    
    /// Called when file metadata changes (e.g., new file loaded)
    fn on_metadata_changed(&mut self, _metadata: &FileMetadata);
    
    /// Name of this subscriber (for debugging)
    fn name(&self) -> &'static str;
    
    /// Show window for this subscriber/panel
    /// Returns Result indicating if there was an error showing the window
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), tramex_tools::errors::TramexError>;
}

/// Central event dispatcher
pub struct EventBus {
    subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
    
    /// Subscribe a new subscriber
    pub fn subscribe(&mut self, subscriber: Box<dyn EventSubscriber>) {
        self.subscribers.push(subscriber);
    }
    
    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
    
    /// Notify all subscribers that a new event was added
    pub fn notify_event_added(&mut self, event: &Trace, index: usize, context: &EventContext) {
        for subscriber in &mut self.subscribers {
            subscriber.on_event_added(event, index, context);
        }
    }
    
    /// Notify all subscribers that an event was focused
    pub fn notify_event_focused(&mut self, event: &Trace, index: usize, context: &EventContext) {
        for subscriber in &mut self.subscribers {
            subscriber.on_event_focused(event, index, context);
        }
    }
    
    /// Notify all subscribers that events were cleared
    pub fn notify_cleared(&mut self) {
        log::debug!("EventBus: Notifying {} subscribers of clear", self.subscribers.len());
        for subscriber in &mut self.subscribers {
            subscriber.on_events_cleared();
        }
    }
    
    /// Notify all subscribers that metadata has changed
    pub fn notify_metadata_changed(&mut self, metadata: &FileMetadata) {
        log::debug!("EventBus: Notifying {} subscribers of metadata change", self.subscribers.len());
        for subscriber in &mut self.subscribers {
            subscriber.on_metadata_changed(metadata);
        }
    }
    
    /// Get subscriber names for tracking open windows
    pub fn subscriber_names(&self) -> Vec<&'static str> {
        self.subscribers.iter().map(|s| s.name()).collect()
    }
    
    /// Show windows for all subscribers
    pub fn show_windows(
        &mut self,
        ctx: &egui::Context,
        open_windows: &std::collections::BTreeSet<String>,
    ) -> Vec<(String, Result<(), tramex_tools::errors::TramexError>)> {
        let mut errors = Vec::new();
        
        for subscriber in &mut self.subscribers {
            let name = subscriber.name().to_owned();
            let mut is_open = open_windows.contains(&name);
            
            if let Err(err) = subscriber.show_window(ctx, &mut is_open) {
                errors.push((name, Err(err)));
            }
        }
        
        errors
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
