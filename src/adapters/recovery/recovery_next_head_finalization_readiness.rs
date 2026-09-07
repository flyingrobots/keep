//! This module owns revalidated next-head finalization readiness.

/// Durable current-state relationship to one finalization request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryNextHeadFinalizationReadiness {
    /// Durable `HEAD` matches the request expectation and the candidate is ready.
    Ready,
    /// Durable `HEAD` already names the exact candidate coordinate.
    AlreadyFinalized,
}
