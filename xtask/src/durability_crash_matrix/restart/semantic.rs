//! This module owns production semantic restart checks for crash states.

use std::error::Error;
use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, CatalogRestartByteLimit, CatalogRestartPolicy, ChecksummedCatalog, ChunkId,
    FilesystemCatalogSnapshot, FilesystemWriterLock, LayoutEntryLimit, RecoveryCatalogStage,
    RecoveryNextHeadStage, RecoverySegmentStage, SegmentReadPolicy, SegmentRecordIdentity,
    SegmentRecordLimit, classify_recovery_catalog_stage, classify_recovery_next_head_stage,
    classify_recovery_segment_stage,
};

use super::expectation::{
    ArtifactBytes, CATALOG_STAGE, ExpectedStoreState, HEAD, NEXT_HEAD, SEGMENT_STAGE, WRITER_LOCK,
};
use crate::durability_crash_matrix::DurabilityCrashMatrixError;
use crate::durability_crash_matrix::state::fixture::{CATALOG_POOL_PATH, SEGMENT_POOL_PATH};

const RESTART_BYTE_LIMIT: u64 = 1_048_576;

pub(super) fn verify(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    verify_writer_release(store_root, expected)?;
    verify_segment_stage(store_root, expected)?;
    verify_catalog_stage(store_root, expected)?;
    verify_next_head(store_root, expected)?;
    verify_immutable_artifacts(store_root, expected)?;
    verify_published_snapshot(store_root, expected)
}

fn verify_writer_release(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    if expected.artifact(WRITER_LOCK).is_none() {
        return Ok(());
    }
    let lock = FilesystemWriterLock::try_acquire(store_root)
        .map_err(|source| verification("reacquire writer lock after process death", source))?;
    drop(lock);
    Ok(())
}

fn verify_segment_stage(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    let Some(bytes) = expected.artifact(SEGMENT_STAGE) else {
        return Ok(());
    };
    let encoded = fs::read(store_root.join(SEGMENT_STAGE))
        .map_err(|source| DurabilityCrashMatrixError::io("read recovery segment stage", source))?;
    let observed = match classify_recovery_segment_stage(&encoded, segment_policy())
        .map_err(|source| verification("classify recovery segment stage", source))?
    {
        RecoverySegmentStage::Reusable(_) => "reusable",
        RecoverySegmentStage::Complete(_) => "complete",
        RecoverySegmentStage::Truncated(_) => "truncated",
    };
    let expected_class = match bytes {
        ArtifactBytes::Segment(64 | 209) => "reusable",
        ArtifactBytes::Segment(337) => "complete",
        ArtifactBytes::Empty | ArtifactBytes::Segment(_) => "truncated",
        ArtifactBytes::Catalog(_) | ArtifactBytes::Head(_) => {
            return Err(DurabilityCrashMatrixError::UnexpectedArtifactKind {
                artifact: SEGMENT_STAGE,
                expected: "segment",
                observed: bytes.kind(),
            });
        }
    };
    require_class(observed, expected_class)
}

fn verify_catalog_stage(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    let Some(bytes) = expected.artifact(CATALOG_STAGE) else {
        return Ok(());
    };
    let encoded = fs::read(store_root.join(CATALOG_STAGE))
        .map_err(|source| DurabilityCrashMatrixError::io("read recovery catalog stage", source))?;
    let observed = match classify_recovery_catalog_stage(&encoded)
        .map_err(|source| verification("classify recovery catalog stage", source))?
    {
        RecoveryCatalogStage::Complete(_) => "complete",
        RecoveryCatalogStage::HeaderTruncated { .. }
        | RecoveryCatalogStage::BodyTruncated { .. } => "truncated",
    };
    let expected_class = match bytes {
        ArtifactBytes::Catalog(352) => "complete",
        ArtifactBytes::Empty | ArtifactBytes::Catalog(_) => "truncated",
        ArtifactBytes::Segment(_) | ArtifactBytes::Head(_) => {
            return Err(DurabilityCrashMatrixError::UnexpectedArtifactKind {
                artifact: CATALOG_STAGE,
                expected: "catalog",
                observed: bytes.kind(),
            });
        }
    };
    require_class(observed, expected_class)
}

fn verify_next_head(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    let Some(bytes) = expected.artifact(NEXT_HEAD) else {
        return Ok(());
    };
    let encoded = fs::read(store_root.join(NEXT_HEAD))
        .map_err(|source| DurabilityCrashMatrixError::io("read recovery next head", source))?;
    let observed = match classify_recovery_next_head_stage(&encoded)
        .map_err(|source| verification("classify recovery next head", source))?
    {
        RecoveryNextHeadStage::Complete(_) => "complete",
        RecoveryNextHeadStage::Truncated { .. } => "truncated",
    };
    let expected_class = match bytes {
        ArtifactBytes::Head(128) => "complete",
        ArtifactBytes::Empty | ArtifactBytes::Head(_) => "truncated",
        ArtifactBytes::Segment(_) | ArtifactBytes::Catalog(_) => {
            return Err(DurabilityCrashMatrixError::UnexpectedArtifactKind {
                artifact: NEXT_HEAD,
                expected: "head",
                observed: bytes.kind(),
            });
        }
    };
    require_class(observed, expected_class)
}

fn verify_immutable_artifacts(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    let Some(_) = expected.artifact(SEGMENT_POOL_PATH) else {
        return Ok(());
    };
    let segment_bytes = fs::read(store_root.join(SEGMENT_POOL_PATH))
        .map_err(|source| DurabilityCrashMatrixError::io("read immutable segment", source))?;
    let segment = AdmittedSegment::decode(&segment_bytes, segment_policy())
        .map_err(|source| verification("admit immutable segment after restart", source))?;
    if expected.artifact(CATALOG_POOL_PATH).is_none() {
        return Ok(());
    }
    let catalog_bytes = fs::read(store_root.join(CATALOG_POOL_PATH))
        .map_err(|source| DurabilityCrashMatrixError::io("read immutable catalog", source))?;
    let _catalog = ChecksummedCatalog::decode(&catalog_bytes)
        .map_err(|source| verification("decode immutable catalog after restart", source))?
        .admit(&[segment])
        .map_err(|source| verification("admit immutable catalog after restart", source))?;
    Ok(())
}

fn verify_published_snapshot(
    store_root: &Path,
    expected: &ExpectedStoreState,
) -> Result<(), DurabilityCrashMatrixError> {
    if expected.artifact(HEAD).is_none() {
        return Ok(());
    }
    let loaded = FilesystemCatalogSnapshot::load(store_root, restart_policy()?)
        .map_err(|source| verification("load published restart snapshot", source))?;
    let observed_generation = loaded.generation().get();
    if observed_generation != 1 {
        return Err(DurabilityCrashMatrixError::SnapshotGenerationMismatch {
            expected: 1,
            observed: observed_generation,
        });
    }
    let snapshot = loaded
        .snapshot()
        .map_err(|source| verification("admit published restart snapshot", source))?;
    let chunk = ChunkId::hash_bytes(&[0])
        .map_err(|source| verification("construct Golden Worldline chunk identity", source))?;
    let record = snapshot.record(SegmentRecordIdentity::Chunk(chunk)).ok_or(
        DurabilityCrashMatrixError::MissingVisibleRecord {
            record: "one-zero chunk",
        },
    )?;
    if record.payload() == [0] {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::artifact_bytes(
            "visible one-zero record payload",
            &[0],
            record.payload(),
        ))
    }
}

const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn restart_policy() -> Result<CatalogRestartPolicy, DurabilityCrashMatrixError> {
    let byte_limit = CatalogRestartByteLimit::new(RESTART_BYTE_LIMIT)
        .map_err(|source| verification("construct restart byte limit", source))?;
    Ok(CatalogRestartPolicy::new(segment_policy(), byte_limit))
}

fn require_class(
    observed: &'static str,
    expected: &'static str,
) -> Result<(), DurabilityCrashMatrixError> {
    if observed == expected {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::ArtifactClassificationMismatch { expected, observed })
    }
}

fn verification(phase: &'static str, source: impl Error + 'static) -> DurabilityCrashMatrixError {
    DurabilityCrashMatrixError::Verification {
        phase,
        source: Box::new(source),
    }
}

#[cfg(test)]
mod tests {
    use super::require_class;

    #[test]
    fn classification_mismatch_names_expected_and_observed() -> Result<(), &'static str> {
        let error = require_class("truncated", "complete")
            .err()
            .ok_or("mismatched classes were accepted")?;

        assert_eq!(
            error.to_string(),
            "post-crash artifact classification mismatch: expected `complete`, observed `truncated`"
        );
        Ok(())
    }
}
