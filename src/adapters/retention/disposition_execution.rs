//! This module owns ordered execution of one disposition's durable phases.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoveryDispositionPhase, RecoveryDispositionStorage};
use crate::adapters::RecoveryDispositionDecision;

/// What one disposition run executed.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionExecutionReceipt {
    executed: Vec<RecoveryDispositionPhase>,
}

impl RecoveryDispositionExecutionReceipt {
    /// The phases executed, in order; empty when the residue was complete.
    #[must_use]
    pub fn executed(&self) -> &[RecoveryDispositionPhase] {
        &self.executed
    }
}

/// A disposition phase that refused, with the phases completed before it.
#[derive(Debug)]
pub struct RecoveryDispositionError {
    phase: RecoveryDispositionPhase,
    executed: Vec<RecoveryDispositionPhase>,
    source: io::Error,
}

impl RecoveryDispositionError {
    /// The refused phase.
    #[must_use]
    pub const fn phase(&self) -> RecoveryDispositionPhase {
        self.phase
    }

    /// The phases completed before the refusal.
    #[must_use]
    pub fn executed(&self) -> &[RecoveryDispositionPhase] {
        &self.executed
    }
}

impl fmt::Display for RecoveryDispositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "disposition phase {:?} refused after {} completed phases",
            self.phase,
            self.executed.len()
        )
    }
}

impl Error for RecoveryDispositionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Executes every phase of `decision` from `from` onwards, in order.
///
/// A refused phase leaves the completed phases' effects in place and names
/// itself; the caller re-observes the residue and resumes rather than
/// continuing from stale evidence.
///
/// # Errors
///
/// Returns [`RecoveryDispositionError`] with the refused phase, the phases
/// completed before it, and the storage's own error as source.
pub fn resume_recovery_disposition<S: RecoveryDispositionStorage>(
    storage: &mut S,
    decision: RecoveryDispositionDecision,
    from: RecoveryDispositionPhase,
) -> Result<RecoveryDispositionExecutionReceipt, RecoveryDispositionError> {
    let phases = RecoveryDispositionPhase::for_decision(decision);
    let start = phases
        .iter()
        .position(|phase| *phase == from)
        .unwrap_or(phases.len());
    let mut executed = Vec::new();
    for phase in phases.iter().copied().skip(start) {
        let result = match phase {
            RecoveryDispositionPhase::WriteStage => storage.write_disposition_stage(),
            RecoveryDispositionPhase::SynchronizeStage => storage.synchronize_disposition_stage(),
            RecoveryDispositionPhase::LinkReceipt => storage.link_disposition_receipt(),
            RecoveryDispositionPhase::SynchronizeDispositions => storage.synchronize_dispositions(),
            RecoveryDispositionPhase::RemoveStage => storage.remove_disposition_stage(),
            RecoveryDispositionPhase::SynchronizeRecovery => storage.synchronize_recovery(),
            RecoveryDispositionPhase::RemoveRetainedStage => storage.remove_retained_stage(),
            RecoveryDispositionPhase::SynchronizeRetention => {
                storage.synchronize_retention_after_disposition()
            }
            RecoveryDispositionPhase::RemovePoolEntry => storage.remove_pool_entry(),
            RecoveryDispositionPhase::SynchronizePool => storage.synchronize_pool(),
        };
        if let Err(source) = result {
            return Err(RecoveryDispositionError {
                phase,
                executed,
                source,
            });
        }
        executed.push(phase);
    }
    Ok(RecoveryDispositionExecutionReceipt { executed })
}

/// Executes every phase of a fresh disposition under `decision`.
///
/// # Errors
///
/// As [`resume_recovery_disposition`].
pub fn execute_recovery_disposition<S: RecoveryDispositionStorage>(
    storage: &mut S,
    decision: RecoveryDispositionDecision,
) -> Result<RecoveryDispositionExecutionReceipt, RecoveryDispositionError> {
    resume_recovery_disposition(storage, decision, RecoveryDispositionPhase::WriteStage)
}
