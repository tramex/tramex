//! Event bus for coordinating notifications to subscribers

use egui;
use tramex_tools::data::Trace;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;

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

/// UI grouping for subscriber panels
///
/// Panels in the `Main` group are rendered inline in fixed areas of the
/// central view; all other groups correspond to floating windows that can be
/// shown or hidden together via the toolbar toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelGroup {
    /// Fixed central view (e.g. the Messages panel)
    Main,
    /// RRC-related floating windows
    Rrc,
    /// NAS-related floating windows
    Nas,
    /// Radio-related floating windows
    Radio,
}

impl PanelGroup {
    /// Groups rendered as floating windows (all except `Main`)
    pub const WINDOWED: [PanelGroup; 3] = [PanelGroup::Rrc, PanelGroup::Nas, PanelGroup::Radio];

    /// Human-readable label for the group
    pub fn label(self) -> &'static str {
        match self {
            PanelGroup::Main => "Main",
            PanelGroup::Rrc => "RRC",
            PanelGroup::Nas => "NAS",
            PanelGroup::Radio => "Radio",
        }
    }
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

    /// Panel group this subscriber belongs to
    fn group(&self) -> PanelGroup;

    /// Render this panel inside a fixed area of the UI
    ///
    /// Used by `Main`-group panels that are not shown as floating windows.
    /// Default implementation does nothing.
    fn show_inline(&mut self, _ui: &mut egui::Ui) {}

    /// Show window for this subscriber/panel
    /// Returns Result indicating if there was an error showing the window
    ///
    /// # Errors
    ///
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), tramex_tools::errors::TramexError>;

    #[cfg(feature = "ai")]
    /// Set the AI API key, provider, and model for panels that support AI features
    fn set_ai_config(&mut self, _key: &str, _provider: &tramex_tools::ai::AIProvider, _model: &str) {}
}

/// Central event dispatcher
pub struct EventBus {
    /// Subscribers list
    subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self { subscribers: Vec::new() }
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
        log::debug!(
            "EventBus: Notifying {} subscribers of metadata change",
            self.subscribers.len()
        );
        for subscriber in &mut self.subscribers {
            subscriber.on_metadata_changed(metadata);
        }
    }

    /// Get subscriber groups and names for tracking open windows
    ///
    /// Returns all subscribers except `Main`-group panels (rendered inline).
    pub fn subscriber_groups(&self) -> Vec<(PanelGroup, &'static str)> {
        self.subscribers
            .iter()
            .filter(|s| s.group() != PanelGroup::Main)
            .map(|s| (s.group(), s.name()))
            .collect()
    }

    /// Show windows for all subscribers
    /// Returns a list of (panel_name, is_open, result) so the caller can update open_windows
    pub fn show_windows(
        &mut self,
        ctx: &egui::Context,
        open_windows: &std::collections::BTreeSet<String>,
    ) -> Vec<(String, bool, Result<(), tramex_tools::errors::TramexError>)> {
        let mut results = Vec::new();

        for subscriber in &mut self.subscribers {
            if subscriber.group() == PanelGroup::Main {
                continue;
            }
            let name = subscriber.name().to_owned();
            let mut is_open = open_windows.contains(&name);

            let result = subscriber.show_window(ctx, &mut is_open);
            results.push((name, is_open, result));
        }

        results
    }

    /// Render all `Main`-group subscribers inline inside the given UI
    pub fn show_main(&mut self, ui: &mut egui::Ui) {
        for subscriber in &mut self.subscribers {
            if subscriber.group() == PanelGroup::Main {
                subscriber.show_inline(ui);
            }
        }
    }

    #[cfg(feature = "ai")]
    /// Forward AI config to all subscribers
    pub fn set_ai_config(&mut self, key: &str, provider: &tramex_tools::ai::AIProvider, model: &str) {
        for subscriber in &mut self.subscribers {
            subscriber.set_ai_config(key, provider, model);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
