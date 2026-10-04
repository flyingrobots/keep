//! This module owns idempotent fixed-stage removal outcomes.

/// Namespace state observed by exact-evidence stage removal.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageDiscardOutcome {
    /// The exact fingerprint-bound stage was removed.
    Removed,
    /// The canonical stage name was already absent.
    AlreadyAbsent,
}
