//! Association module for parent/child trace relationships
//! 
//! This module provides the logic for associating traces that are logically related
//! (e.g., NAS messages encapsulated in RRC messages).

mod relation;
mod rules;
mod matcher;

pub use relation::{TraceRelation, AssociationStatus};
pub use rules::{AssociationRule, SearchDirection, AssociationRules, NasToRrcRule};
pub use matcher::TraceMatcher;

use crate::data::Trace;

/// Default lookback window for cross-batch association computation
pub const ASSOCIATION_LOOKBACK_WINDOW: usize = 10;

/// Compute associations for traces starting from a given index.
/// This is a standalone function that can be used by any code that has a `&mut [Trace]`.
/// 
/// # Arguments
/// * `events` - Mutable slice of traces to compute associations for
/// * `rules` - The association rules to use for matching
/// * `start_index` - Index to start computing from (useful for incremental batch processing)
/// 
/// # Note
/// When processing new batches, pass `start_index = max(0, new_batch_start - LOOKBACK_WINDOW)` 
/// to handle cases where parent/child relationships span across batches.
pub fn compute_associations(events: &mut [Trace], rules: &AssociationRules, start_index: usize) {
    let trace_count = events.len();
    
    for index in start_index..trace_count {
        // Skip if already computed (unless we're in the lookback window - force recompute)
        let in_lookback = index < start_index + ASSOCIATION_LOOKBACK_WINDOW;
        if !in_lookback && events.get(index).map(|t| t.relation.parent.is_computed()).unwrap_or(true) {
            continue;
        }
        
        // Reset relation if we're recomputing in lookback window
        if in_lookback {
            if let Some(trace) = events.get_mut(index) {
                // Only reset if it was NotFound (might find match in new batch)
                if matches!(trace.relation.parent, AssociationStatus::NotFound) {
                    trace.relation.parent = AssociationStatus::NotComputed;
                }
            }
        }
        
        // Get the trace's layer
        let layer = match events.get(index) {
            Some(t) => t.layer.clone(),
            None => continue,
        };
        
        // Find applicable rules for this layer
        let applicable_rules = rules.rules_for_layer(&layer);
        
        if applicable_rules.is_empty() {
            // No rules for this layer - mark as not applicable
            if let Some(trace) = events.get_mut(index) {
                trace.relation.set_parent_not_applicable();
            }
            continue;
        }
        
        // Try each applicable rule
        let mut found = false;
        for rule in applicable_rules {
            let status = TraceMatcher::find_relative(
                index, 
                events, 
                rule, 
                &layer, 
                &rule.target_layer()
            );
            
            // Update the source trace's parent relation
            if let Some(trace) = events.get_mut(index) {
                trace.relation.parent = status.clone();
            }
            
            // If found, also set the child relation on the target trace
            if let AssociationStatus::Found(target_idx) = status {
                if let Some(target_trace) = events.get_mut(target_idx) {
                    target_trace.relation.set_child(index);
                }
                found = true;
                break; // Stop after first match
            }
        }
        
        // Mark as not found if no rules matched
        if !found {
            if let Some(trace) = events.get_mut(index) {
                if !trace.relation.parent.is_computed() {
                    trace.relation.parent = AssociationStatus::NotFound;
                }
            }
        }
    }
}
