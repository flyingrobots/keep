//! This module owns observable next-head finalization outcomes.

/// How one exact recovery next head reached the durable current coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryNextHeadFinalizationOutcome {
    /// The exact candidate replaced the prior durable head.
    Finalized,
    /// The exact candidate was already current during retry.
    AlreadyFinalized,
}
