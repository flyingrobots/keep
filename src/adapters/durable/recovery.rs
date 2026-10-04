//! This boundary module owns recovery of an interrupted durable ingestion:
//! the version-one recovery protocol over the store's staging residue, with
//! ingestion's own evidence for a complete stage.

use std::path::Path;

use crate::adapters::compaction::{
    CompactionRecovery, CompleteStageEvidence, FilesystemCompactionRecoveryError, recover_with,
};
use crate::adapters::{CatalogRestartPolicy, FilesystemRecoveryStageDiscarder};

/// Recovers an interrupted ingestion on the version-two root at
/// `store_root`, acquiring writer authority for the duration.
///
/// A truncated `staging/current.seg` is discarded through the version-one
/// protocol. A complete one is discarded when the current catalog names
/// none of its records: it is an ingestion stage that never reached
/// commit, so nothing published references it. A retained
/// `staging/current.cat` or `head.next` resolves exactly as after an
/// interrupted compaction, since commit runs the same catalog protocol.
/// The report and error are shared with compaction recovery for the same
/// reason.
///
/// # Errors
///
/// Returns [`FilesystemCompactionRecoveryError`] at the exact open,
/// assessment, planning, or execution refusal, including a complete stage
/// the current catalog partly names, which is neither residue this
/// protocol recognizes. Admission refusals initiate no recovery mutation. Execution
/// errors expose completed earlier actions and the failing action's known or
/// uncertain effects through `execution()`; failure does not imply rollback.
/// Recovery stops at the first error, and a subsequent call observes the store anew.
pub fn recover_durable_ingestion(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store_root)
        .map_err(|source| FilesystemCompactionRecoveryError::refused("open", source))?;
    recover_with(discarder, policy, CompleteStageEvidence::Unpublished)
}

#[cfg(test)]
pub(super) fn recover_durable_ingestion_unchecked_for_tests(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder =
        FilesystemRecoveryStageDiscarder::open_unchecked_version_two_for_tests(store_root)
            .map_err(|source| FilesystemCompactionRecoveryError::refused("open", source))?;
    recover_with(discarder, policy, CompleteStageEvidence::Unpublished)
}
