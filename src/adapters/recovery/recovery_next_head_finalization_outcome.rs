//! This module owns observable next-head finalization outcomes.

/// Namespace outcome of one exact recovery next head.
///
/// Durability requires a successful finalization receipt. A root-sync error may
/// carry this same outcome without confirming directory durability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryNextHeadFinalizationOutcome {
    /// This invocation replaced the prior head with the exact candidate.
    Finalized,
    /// The exact candidate was already current during retry.
    AlreadyFinalized,
}
