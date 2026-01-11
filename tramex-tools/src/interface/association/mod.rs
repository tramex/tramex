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
        // Check if we're in the lookback window for recomputation
        let in_lookback = index < start_index + ASSOCIATION_LOOKBACK_WINDOW;
        
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
            
            // If found, set parent/child based on rule's direction
            if let AssociationStatus::Found(target_indices) = status {
                for &target_idx in &target_indices {
                    if rule.source_is_child() {
                        // Source is child, target is parent (e.g., NAS→RRC)
                        if let Some(trace) = events.get_mut(index) {
                            trace.relation.add_parent(target_idx);
                        }
                        if let Some(target_trace) = events.get_mut(target_idx) {
                            target_trace.relation.add_child(index);
                        }
                    } else {
                        // Source is parent, target is child (e.g., NGAP→NAS)
                        if let Some(trace) = events.get_mut(index) {
                            trace.relation.add_child(target_idx);
                        }
                        if let Some(target_trace) = events.get_mut(target_idx) {
                            target_trace.relation.add_parent(index);
                        }
                    }
                }
                found = true;
                // Don't break - continue to run other rules for additional associations
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
