//! This boundary module owns ordered GC execution from any resumption point.

use std::error::Error;
use std::fmt;
use std::io;

use crate::adapters::retention::{RetentionStorageError, RetentionStorageProgress};

use super::{GcExecutionPhase, GcExecutionPoint, GcExecutionStorage};

/// What one execution run executed.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcExecutionReceipt {
    executed: Vec<GcExecutionPoint>,
}

impl GcExecutionReceipt {
    /// The points executed, in order; empty when the residue was complete.
    #[must_use]
    pub fn executed(&self) -> &[GcExecutionPoint] {
        &self.executed
    }
}

/// An execution failure, with successful earlier calls and the original cause.
///
/// A failed capability may already have changed the namespace; this is not rollback.
#[derive(Debug)]
pub struct GcExecutionError {
    point: GcExecutionPoint,
    executed: Vec<GcExecutionPoint>,
    source: io::Error,
}

impl GcExecutionError {
    /// Progress reported by the failing storage capability.
    ///
    /// `None` means unreported effects, never absence of effects. Earlier
    /// successful calls are listed separately by [`Self::executed`].
    #[must_use]
    pub fn storage_progress(&self) -> Option<&RetentionStorageProgress> {
        self.source
            .get_ref()
            .and_then(|source| source.downcast_ref::<RetentionStorageError>())
            .and_then(RetentionStorageError::progress)
    }

    /// The refused point.
    #[must_use]
    pub const fn point(&self) -> GcExecutionPoint {
        self.point
    }

    /// Earlier capability calls that returned success, in order.
    ///
    /// Success here does not imply a new namespace effect or directory durability.
    /// In particular, a link may already exist and its sync is a separate call.
    /// Effects inside the failing call are reported by [`Self::storage_progress`].
    #[must_use]
    pub fn executed(&self) -> &[GcExecutionPoint] {
        &self.executed
    }
}

impl fmt::Display for GcExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} failed for candidate {} after {} successful calls",
            self.point.phase,
            self.point.candidate,
            self.executed.len()
        )
    }
}

impl Error for GcExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Expands the phase order for `candidate_count` candidates into points.
fn points(candidate_count: usize) -> Vec<GcExecutionPoint> {
    let mut points = Vec::new();
    for phase in GcExecutionPhase::ALL {
        if phase == GcExecutionPhase::UnlinkCandidate {
            for candidate in 0..candidate_count {
                points.push(GcExecutionPoint {
                    phase: GcExecutionPhase::UnlinkCandidate,
                    candidate,
                });
                points.push(GcExecutionPoint {
                    phase: GcExecutionPhase::SynchronizeSegmentPool,
                    candidate,
                });
            }
        } else if phase != GcExecutionPhase::SynchronizeSegmentPool {
            points.push(GcExecutionPoint::at(phase));
        }
    }
    points
}

/// Executes every point from `from` onwards for `candidate_count`
/// candidates, in order.
///
/// Execution stops at the first error. Earlier successful calls and the failing
/// call may have effects; a failed synchronization never rolls them back.
/// The caller must reobserve residue before another attempt. The completed-call
/// list does not itself certify effects or durability; inspect the failing
/// capability report when supplied by the storage implementation.
///
/// # Errors
///
/// Returns [`GcExecutionError`] with the refused point, the points completed
/// before it, and the storage's own error as source.
pub fn resume_gc_execution<S: GcExecutionStorage>(
    storage: &mut S,
    candidate_count: usize,
    from: GcExecutionPoint,
) -> Result<GcExecutionReceipt, GcExecutionError> {
    let all = points(candidate_count);
    let start = all
        .iter()
        .position(|point| *point == from)
        .unwrap_or(all.len());
    let mut executed = Vec::new();
    for point in all.into_iter().skip(start) {
        let result = match point.phase {
            GcExecutionPhase::WriteIntentStage => storage.write_intent_stage(),
            GcExecutionPhase::SynchronizeIntentStage => storage.synchronize_intent_stage(),
            GcExecutionPhase::LinkIntent => storage.link_intent(),
            GcExecutionPhase::SynchronizeGcAfterIntent => storage.synchronize_gc_after_intent(),
            GcExecutionPhase::RemoveIntentStage => storage.remove_intent_stage(),
            GcExecutionPhase::SynchronizeGcAfterIntentCleanup => {
                storage.synchronize_gc_after_intent_cleanup()
            }
            GcExecutionPhase::UnlinkCandidate => storage.unlink_candidate(point.candidate),
            GcExecutionPhase::SynchronizeSegmentPool => {
                storage.synchronize_segment_pool(point.candidate)
            }
            GcExecutionPhase::WriteReceiptStage => storage.write_receipt_stage(),
            GcExecutionPhase::SynchronizeReceiptStage => storage.synchronize_receipt_stage(),
            GcExecutionPhase::ReplaceReceipt => storage.replace_receipt(),
            GcExecutionPhase::SynchronizeGcAfterReceipt => storage.synchronize_gc_after_receipt(),
            GcExecutionPhase::RemoveIntent => storage.remove_intent(),
            GcExecutionPhase::SynchronizeGcAfterIntentRemoval => {
                storage.synchronize_gc_after_intent_removal()
            }
        };
        if let Err(source) = result {
            return Err(GcExecutionError {
                point,
                executed,
                source,
            });
        }
        executed.push(point);
    }
    Ok(GcExecutionReceipt { executed })
}

/// Executes a fresh retirement of `candidate_count` candidates.
///
/// # Errors
///
/// As [`resume_gc_execution`].
pub fn execute_gc<S: GcExecutionStorage>(
    storage: &mut S,
    candidate_count: usize,
) -> Result<GcExecutionReceipt, GcExecutionError> {
    resume_gc_execution(storage, candidate_count, GcExecutionPoint::START)
}
