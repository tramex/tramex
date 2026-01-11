//! Trace matcher for finding related traces

use super::relation::AssociationStatus;
use super::rules::{AssociationRule, SearchDirection};
use crate::data::Trace;
use crate::interface::layer::Layer;

/// Matcher for finding related trace relationships
/// Provides functionality to search for related traces (parent/child relationships)
/// based on configurable association rules and search directions.
pub struct TraceMatcher;

impl TraceMatcher {
    /// Find a relative trace using the rule's source and target layers
    /// This is the main public API - the rule defines what we're searching for
    /// 
    /// # Arguments
    /// * `source_index` - Index of the source trace in the events vector
    /// * `events` - Reference to all traces
    /// * `rule` - The association rule to use for matching
    /// * `source_layer` - The layer the source trace should have
    /// * `target_layer` - The layer we're searching for
    pub fn find_relative(
        source_index: usize,
        events: &[Trace],
        rule: &dyn AssociationRule,
        source_layer: &Layer,
        target_layer: &Layer,
    ) -> AssociationStatus {
        let source = match events.get(source_index) {
            Some(t) => t,
            None => return AssociationStatus::NotFound,
        };

        // Check if this trace's layer matches the expected source layer
        if &source.layer != source_layer {
            return AssociationStatus::NotApplicable;
        }

        let direction = rule.preferred_direction(source);
        
        // Search based on preferred direction
        match direction {
            SearchDirection::BackwardFirst => {
                if let Some(idx) = Self::search_backward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
                if let Some(idx) = Self::search_forward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
            }
            SearchDirection::ForwardFirst => {
                if let Some(idx) = Self::search_forward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
                if let Some(idx) = Self::search_backward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
            }
            SearchDirection::BackwardOnly => {
                if let Some(idx) = Self::search_backward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
            }
            SearchDirection::ForwardOnly => {
                if let Some(idx) = Self::search_forward(source_index, events, source, rule, &target_layer) {
                    return AssociationStatus::Found(vec![idx]);
                }
            }
        }

        AssociationStatus::NotFound
    }

    /// Find a relative trace with swapped source/target layers (reverse search)
    pub fn find_relative_reverse(source_index: usize, events: &[Trace], rule: &dyn AssociationRule) -> AssociationStatus {
        Self::find_relative(source_index, events, rule, &rule.target_layer(), &rule.source_layer())
    }

    /// Search backward (previous traces) for a matching trace
    fn search_backward(
        index: usize,
        events: &[Trace],
        source: &Trace,
        rule: &dyn AssociationRule,
        target_layer: &Layer,
    ) -> Option<usize> {
        let window_size = rule.window_size();
        let start = index.saturating_sub(window_size);
        for idx in (start..index).rev() {
            let candidate = &events[idx];

            if &candidate.layer != target_layer {
                continue;
            }

            if rule.matches(source, candidate) {
                return Some(idx);
            }
        }
        None
    }

    /// Search forward (next traces) for a matching trace
    fn search_forward(
        index: usize,
        events: &[Trace],
        source: &Trace,
        rule: &dyn AssociationRule,
        target_layer: &Layer,
    ) -> Option<usize> {
        let window_size = rule.window_size();
        let end = (index + window_size + 1).min(events.len());
        for idx in (index + 1)..end {
            let candidate = &events[idx];

            if &candidate.layer != target_layer {
                continue;
            }

            if rule.matches(source, candidate) {
                return Some(idx);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_direction_from_direction() {
        use crate::interface::types::Direction;

        assert_eq!(
            SearchDirection::from_direction(&Direction::UL),
            SearchDirection::BackwardFirst
        );
        assert_eq!(
            SearchDirection::from_direction(&Direction::DL), 
            SearchDirection::ForwardFirst
        );
    }
}
