//! This module owns semantic parent directories for fixed recovery stages.

/// Protocol parent directory selected by one canonical fixed-stage name.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageParent {
    /// The `staging` protocol directory.
    Staging,
    /// The store root.
    Root,
}
