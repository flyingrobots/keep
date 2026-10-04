//! This boundary module owns the exact read of `gc` and the segment pool
//! that restart classifies, and the names GC execution writes.

use std::io;

use cap_std::fs::Dir;

use super::{AdmittedGcRetirementIntent, GcResidue, GcRetirementIntent};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::filesystem_stage_observation::StageObservation;
use crate::adapters::physical_pool_name;

/// `gc/intent.next`.
pub(super) const INTENT_STAGE: &str = "intent.next";
/// `gc/intent`.
pub(super) const INTENT: &str = "intent";
/// `gc/receipt.next`.
pub(super) const RECEIPT_STAGE: &str = "receipt.next";
/// `gc/receipt`.
pub(super) const RECEIPT: &str = "receipt";
/// Every entry `gc` may hold.
pub(in crate::adapters) const GC_ENTRY_NAMES: [&str; 4] =
    [INTENT_STAGE, INTENT, RECEIPT_STAGE, RECEIPT];

const RECEIPT_LENGTH: usize = 320;

/// Reads every `gc` record as it is, bounded one byte past its maximum
/// length, and the presence of every candidate the durable intent names.
/// Filesystem evidence stays alive beside the planner's semantic residue.
pub(super) struct GcResidueObservation {
    pub(super) residue: GcResidue,
    pub(super) intent_stage: Option<StageObservation>,
    pub(super) receipt_stage: Option<StageObservation>,
}

pub(super) fn read(gc: &Dir, segments: &Dir) -> io::Result<GcResidueObservation> {
    let intent_bound =
        super::intent_format::canonical_length(GcRetirementIntent::MAXIMUM_CANDIDATE_COUNT)
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| invalid(super::FilesystemGcRefusal::IntentBoundOverflow))?;
    let receipt_bound = RECEIPT_LENGTH.saturating_add(1);
    let intent = read_bounded(gc, INTENT, intent_bound)?;
    let candidates_present = match intent.as_deref().map(AdmittedGcRetirementIntent::decode) {
        Some(Ok(admitted)) => candidates_present(segments, admitted.intent())?,
        Some(Err(_)) | None => Vec::new(),
    };
    let intent_stage = StageObservation::read(gc, INTENT_STAGE, intent_bound)?;
    let receipt_stage = StageObservation::read(gc, RECEIPT_STAGE, receipt_bound)?;
    let residue = GcResidue {
        intent_stage: intent_stage.as_ref().map(|stage| Box::from(stage.bytes())),
        intent,
        receipt_stage: receipt_stage.as_ref().map(|stage| Box::from(stage.bytes())),
        receipt: read_bounded(gc, RECEIPT, receipt_bound)?,
        candidates_present,
    };
    Ok(GcResidueObservation {
        residue,
        intent_stage,
        receipt_stage,
    })
}

/// Whether each candidate's pool entry exists, by name, without following
/// links; execution verifies bytes, restart only counts presence.
fn candidates_present(segments: &Dir, intent: &GcRetirementIntent) -> io::Result<Vec<bool>> {
    let mut present = Vec::new();
    for candidate in intent.candidates() {
        let name = physical_pool_name::segment(candidate.segment_digest());
        match segments.symlink_metadata(&name) {
            Ok(_metadata) => present.push(true),
            Err(source) if source.kind() == io::ErrorKind::NotFound => present.push(false),
            Err(source) => return Err(source),
        }
    }
    Ok(present)
}

fn read_bounded(gc: &Dir, name: &str, bound: usize) -> io::Result<Option<Box<[u8]>>> {
    exact_record::read_bounded_optional(gc, name, bound)
        .map(|bytes| bytes.map(Vec::into_boxed_slice))
        .map_err(ExactRecordError::into_io)
}

/// Removes a discardable stage and proves it gone.
pub(super) fn discard(gc: &Dir, name: &str) -> io::Result<()> {
    gc.remove_file(name)?;
    exact_record::require_absent(gc, name).map_err(ExactRecordError::into_io)?;
    crate::adapters::filesystem_catalog_artifact::synchronize_directory(gc)
}

pub(super) fn invalid(refusal: super::FilesystemGcRefusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}

pub(super) fn invalid_from(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use super::{ExactRecordError, INTENT, read_bounded};
    use crate::adapters::filesystem_exact_record::ExactRecordRefusal;
    use crate::adapters::filesystem_test_sandbox::TestDirectory;

    #[test]
    fn a_non_regular_gc_record_retains_its_exact_refusal() -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create("gc-residue-typed-kind")?;
        fs::create_dir(sandbox.path().join(INTENT))?;
        let directory =
            cap_std::fs::Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
        let error = read_bounded(&directory, INTENT, 1)
            .err()
            .ok_or("non-regular GC intent admitted")?;
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(matches!(
            error
                .get_ref()
                .and_then(|source| source.downcast_ref::<ExactRecordError>()),
            Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
        ));
        drop(directory);
        sandbox.remove()?;
        Ok(())
    }
}
