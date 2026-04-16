//! Event store - single source of truth for all events

use std::ops::Range;
use tramex_tools::data::Trace;
use tramex_tools::interface::association::AssociationRules;

/// Core data store that owns all events
pub struct EventStore {
    /// All events
    events: Vec<Trace>,
    /// Current focused event index
    current_index: usize,
    /// Total number of events (if known, e.g., from file size)
    total_count: Option<usize>,
    /// Whether all data has been loaded
    is_complete: bool,
}

impl EventStore {
    /// Create a new empty event store
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            current_index: 0,
            total_count: None,
            is_complete: false,
        }
    }

    /// Add new events and return the range of indices they occupy
    pub fn add_events(&mut self, new_events: Vec<Trace>) -> Range<usize> {
        let start = self.events.len();
        self.events.extend(new_events);
        let end = self.events.len();

        log::debug!("EventStore: Added {} events (total now: {})", end - start, end);

        start..end
    }

    /// Set the current focused index
    pub fn set_current(&mut self, index: usize) -> Option<&Trace> {
        if index < self.events.len() {
            self.current_index = index;
            Some(&self.events[index])
        } else {
            None
        }
    }

    /// Get the current focused event
    pub fn current(&self) -> Option<&Trace> {
        self.events.get(self.current_index)
    }

    /// Get current index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get all events
    pub fn events(&self) -> &[Trace] {
        &self.events
    }

    /// Get number of loaded events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Check if we can navigate to next event
    pub fn can_go_next(&self) -> bool {
        self.current_index < self.events.len().saturating_sub(1)
    }

    /// Check if we can navigate to previous event
    pub fn can_go_previous(&self) -> bool {
        self.current_index > 0
    }

    /// Navigate to next event
    pub fn go_next(&mut self) -> Option<&Trace> {
        if self.can_go_next() {
            self.current_index += 1;
            Some(&self.events[self.current_index])
        } else {
            None
        }
    }

    /// Navigate to previous event
    pub fn go_previous(&mut self) -> Option<&Trace> {
        if self.can_go_previous() {
            self.current_index -= 1;
            Some(&self.events[self.current_index])
        } else {
            None
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        log::debug!("EventStore: Clearing {} events", self.events.len());
        self.events.clear();
        self.current_index = 0;
        self.total_count = None;
        self.is_complete = false;
    }

    /// Set total count (for progress tracking)
    pub fn set_total_count(&mut self, count: Option<usize>) {
        self.total_count = count;
    }

    /// Get total count
    pub fn total_count(&self) -> Option<usize> {
        self.total_count
    }

    /// Mark as complete (all data loaded)
    pub fn set_complete(&mut self, complete: bool) {
        self.is_complete = complete;
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Get loading progress (0.0 to 1.0)
    pub fn progress(&self) -> Option<f32> {
        self.total_count.map(|total| {
            if total == 0 {
                1.0
            } else {
                self.events.len() as f32 / total as f32
            }
        })
    }

    /// Compute associations for events starting from a given index
    /// Uses lookback window to handle cross-batch relationships
    ///
    /// # Arguments
    /// * `rules` - The association rules to use
    /// * `batch_start` - Start index of the new batch (lookback is applied automatically)
    pub fn compute_associations(&mut self, rules: &AssociationRules, batch_start: usize) {
        use tramex_tools::interface::association::ASSOCIATION_LOOKBACK_WINDOW;

        // Apply lookback to catch cross-batch relationships
        let start_index = batch_start.saturating_sub(ASSOCIATION_LOOKBACK_WINDOW);

        tramex_tools::interface::association::compute_associations(&mut self.events, rules, start_index);
        log::debug!(
            "EventStore: Computed associations from index {} (batch_start={}, lookback={})",
            start_index,
            batch_start,
            ASSOCIATION_LOOKBACK_WINDOW
        );
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}
