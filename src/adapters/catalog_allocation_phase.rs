//! Bounded catalog-admission allocation phases.

use std::fmt;

/// Bounded temporary or retained allocation attempted by catalog admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogAllocationPhase {
    /// Sorted borrowed index over caller-supplied admitted segments.
    SegmentIndex,
    /// Bounded physical lookup plan over canonical catalog entries.
    EntryPlan,
    /// Logical-identity bindings retained by the admitted catalog.
    RecordBindings,
}

impl fmt::Display for CatalogAllocationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SegmentIndex => formatter.write_str("segment index"),
            Self::EntryPlan => formatter.write_str("entry plan"),
            Self::RecordBindings => formatter.write_str("record bindings"),
        }
    }
}
