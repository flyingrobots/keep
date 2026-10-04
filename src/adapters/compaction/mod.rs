//! Identity-preserving compaction for `keep.segment-store/v2`.
//!
//! Compaction copies every live record of a mixed segment (one the current
//! catalog names with at least one record no retained closure reaches) into
//! one new immutable segment, publishes a catalog successor that names the
//! copies and omits every unreachable record, and revalidates the view. The
//! old segments become `unreachable-superseded` for GC. `BlobId`, `ChunkId`,
//! and `LayoutId` never change: the successor names the same identities at
//! new locations. This module owns the observation, the pure planner, the
//! filesystem authority over the existing catalog publication protocol, and
//! the recovery driver for an interrupted successor.

mod error;
mod filesystem;
#[cfg(test)]
mod filesystem_tests;
#[cfg(test)]
mod interruption_tests;
mod observation;
mod plan;
mod recovery;
mod recovery_execution;
mod recovery_failure;
#[cfg(test)]
mod recovery_head_progress_tests;
#[cfg(all(test, target_os = "linux"))]
mod recovery_identity_tests;
mod recovery_observation;
mod recovery_preflight;
mod recovery_progress;
#[cfg(test)]
mod recovery_progress_tests;
mod recovery_schedule;
#[cfg(all(test, target_os = "linux"))]
mod recovery_uncertain_tests;
#[cfg(test)]
mod test_fixture;

pub use error::FilesystemCompactionError;
pub use filesystem::{CompactionPublish, CompactionReceipt, FilesystemCompactionAuthority};
pub use observation::{CompactionObservation, observe_compaction};
pub use plan::{CompactionPlan, CompactionRefusal, CompactionSegmentDisposition, plan_compaction};
#[cfg(test)]
pub(in crate::adapters) use recovery::recover_compaction_unchecked_for_tests;
pub use recovery::{CompactionRecovery, FilesystemCompactionRecoveryError, recover_compaction};
pub(in crate::adapters) use recovery::{CompleteStageEvidence, recover_with};

pub use recovery_progress::{
    CompactionRecoveryAction, CompactionRecoveryBoundary, CompactionRecoveryEffects,
    CompactionRecoveryExecution,
};
