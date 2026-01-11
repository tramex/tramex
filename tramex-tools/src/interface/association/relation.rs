//! Trace relation structures for parent/child associations

/// Status of a trace association attempt
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AssociationStatus {
    /// Association has not been computed yet
    #[default]
    NotComputed,
    /// Association found - contains indices of related traces (can be multiple)
    Found(Vec<usize>),
    /// Association was searched but no match found within the window
    NotFound,
    /// This trace type does not support associations
    NotApplicable,
}

impl AssociationStatus {
    /// Returns true if the association has been computed (Found, NotFound, or NotApplicable)
    pub fn is_computed(&self) -> bool {
        !matches!(self, AssociationStatus::NotComputed)
    }

    /// Returns the first index if found (for backwards compatibility)
    pub fn get_index(&self) -> Option<usize> {
        match self {
            AssociationStatus::Found(indices) => indices.first().copied(),
            _ => None,
        }
    }

    /// Returns all indices if found
    pub fn get_indices(&self) -> Option<&Vec<usize>> {
        match self {
            AssociationStatus::Found(indices) => Some(indices),
            _ => None,
        }
    }

    /// Add an index to the association
    /// - If NotComputed or NotFound: becomes Found with single index
    /// - If Found: appends index if not already present
    /// - If NotApplicable: no change (returns false)
    pub fn add_index(&mut self, index: usize) -> bool {
        match self {
            AssociationStatus::NotComputed | AssociationStatus::NotFound => {
                *self = AssociationStatus::Found(vec![index]);
                true
            }
            AssociationStatus::Found(indices) => {
                if !indices.contains(&index) {
                    indices.push(index);
                }
                true
            }
            AssociationStatus::NotApplicable => false,
        }
    }

    /// Check if this status contains a specific index
    pub fn contains(&self, index: usize) -> bool {
        match self {
            AssociationStatus::Found(indices) => indices.contains(&index),
            _ => false,
        }
    }
}

/// Holds the parent/child relationship for a trace
#[derive(Debug, Clone, Default)]
pub struct TraceRelation {
    /// Index of the parent trace (e.g., RRC for a NAS message)
    pub parent: AssociationStatus,
    /// Index of the child trace (e.g., NAS for an RRC message with dedicated NAS)
    pub child: AssociationStatus,
}

impl TraceRelation {
    /// Create a new empty relation
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this trace has a parent
    pub fn has_parent(&self) -> bool {
        matches!(self.parent, AssociationStatus::Found(_))
    }

    /// Check if this trace has a child
    pub fn has_child(&self) -> bool {
        matches!(self.child, AssociationStatus::Found(_))
    }

    /// Get parent index if available
    pub fn get_parent_index(&self) -> Option<usize> {
        self.parent.get_index()
    }

    /// Get parent indices if available
    pub fn get_parent_indices(&self) -> Option<&Vec<usize>> {
        self.parent.get_indices()
    }

    /// Get child index if available
    pub fn get_child_index(&self) -> Option<usize> {
        self.child.get_index()
    }

    /// Get child indices if available
    pub fn get_child_indices(&self) -> Option<&Vec<usize>> {
        self.child.get_indices()
    }

    /// Set parent as found (replaces existing)
    pub fn set_parent(&mut self, index: usize) {
        self.parent = AssociationStatus::Found(vec![index]);
    }

    /// Add a parent index (appends to existing if Found)
    pub fn add_parent(&mut self, index: usize) -> bool {
        self.parent.add_index(index)
    }

    /// Set child as found (replaces existing)
    pub fn set_child(&mut self, index: usize) {
        self.child = AssociationStatus::Found(vec![index]);
    }

    /// Add a child index (appends to existing if Found)
    pub fn add_child(&mut self, index: usize) -> bool {
        self.child.add_index(index)
    }

    /// Mark parent search as completed with no result
    pub fn set_parent_not_found(&mut self) {
        self.parent = AssociationStatus::NotFound;
    }

    /// Mark child search as completed with no result
    pub fn set_child_not_found(&mut self) {
        self.child = AssociationStatus::NotFound;
    }

    /// Mark parent as not applicable for this trace type
    pub fn set_parent_not_applicable(&mut self) {
        self.parent = AssociationStatus::NotApplicable;
    }

    /// Mark child as not applicable for this trace type
    pub fn set_child_not_applicable(&mut self) {
        self.child = AssociationStatus::NotApplicable;
    }

    /// Reset the relation (mark as not computed) - useful when new traces arrive
    pub fn invalidate(&mut self) {
        self.parent = AssociationStatus::NotComputed;
        self.child = AssociationStatus::NotComputed;
    }
}
