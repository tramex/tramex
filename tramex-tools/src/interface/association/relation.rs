//! Trace relation structures for parent/child associations

/// Status of a trace association attempt
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AssociationStatus {
    /// Association has not been computed yet
    #[default]
    NotComputed,
    /// Association found - contains the index of the related trace
    Found(usize),
    /// Association was searched but no match found within the window
    NotFound,
    /// This trace type does not support this type of association
    NotApplicable,
}

impl AssociationStatus {
    /// Returns true if the association has been computed (Found or NotFound)
    pub fn is_computed(&self) -> bool {
        !matches!(self, AssociationStatus::NotComputed)
    }

    /// Returns the index if found
    pub fn get_index(&self) -> Option<usize> {
        match self {
            AssociationStatus::Found(idx) => Some(*idx),
            _ => None,
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

    /// Get child index if available
    pub fn get_child_index(&self) -> Option<usize> {
        self.child.get_index()
    }

    /// Set parent as found
    pub fn set_parent(&mut self, index: usize) {
        self.parent = AssociationStatus::Found(index);
    }

    /// Set child as found
    pub fn set_child(&mut self, index: usize) {
        self.child = AssociationStatus::Found(index);
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
