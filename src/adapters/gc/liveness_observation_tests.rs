//! Filesystem GC liveness laws over the migrated fixture store.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::{
    GcLimits, GcLivenessObservationError, GcRetentionState, GcSegmentClassification,
    observe_gc_liveness, plan_gc,
};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, migrated_store, reopen_authority,
};
use crate::adapters::test_support::decode_hex;
use crate::adapters::{FilesystemRetentionSnapshot, ReaderAttemptLimit};
use crate::execute_retention_publication;

pub(super) const ORPHAN_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
pub(super) const ORPHAN_SEGMENT_NAME: &str =
    "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc.seg";

pub(super) fn published_store(
    name: &str,
) -> Result<crate::adapters::filesystem_test_sandbox::TestDirectory, Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(sandbox.path())?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    Ok(sandbox)
}

pub(super) fn observe(root: &Path) -> Result<super::GcLivenessSnapshot, Box<dyn Error>> {
    let view =
        FilesystemRetentionSnapshot::load(root, catalog_policy()?, ReaderAttemptLimit::DEFAULT)?;
    observe_gc_liveness(root, &view, catalog_policy()?).map_err(Into::into)
}

#[test]
fn the_published_fixture_store_plans_its_one_segment_live() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-published")?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert!(matches!(
        snapshot.coordinates().retention(),
        GcRetentionState::Published { .. }
    ));
    assert_eq!(snapshot.retained().len(), 1);
    assert_eq!(plan.segments().len(), 1);
    assert!(plan.segments().values().all(|planned| {
        planned.classification() == GcSegmentClassification::Live { retained_roots: 1 }
    }));
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_unpublished_store_plans_its_named_segment_unreachable_but_not_collectible()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("gc-liveness-empty-retention")?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(snapshot.coordinates().retention(), GcRetentionState::Empty);
    assert!(snapshot.retained().is_empty());
    assert!(
        plan.segments().values().all(|planned| {
            planned.classification() == GcSegmentClassification::NamedUnreachable
        })
    );
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_orphan_pool_segment_is_recovery_protected_and_never_a_candidate() -> Result<(), Box<dyn Error>>
{
    let sandbox = published_store("gc-liveness-orphan")?;
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(plan.segments().len(), 2);
    let protected = plan
        .segments()
        .values()
        .filter(|planned| planned.classification() == GcSegmentClassification::RecoveryProtected)
        .count();
    assert_eq!(protected, 1);
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_corrupt_pool_segment_refuses_observation_before_any_plan() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-corrupt")?;
    let mut orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    let last = orphan.last_mut().ok_or("orphan is empty")?;
    *last ^= 1;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;

    let error = observe(sandbox.path())
        .err()
        .ok_or("a corrupt pool segment was unexpectedly observed")?;

    let error = error
        .downcast::<GcLivenessObservationError>()
        .map_err(|_error| "observation refused with the wrong error type")?;
    assert!(matches!(
        *error,
        GcLivenessObservationError::SegmentAdmission { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_pool_entry_not_named_by_a_digest_refuses_observation() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-stray")?;
    fs::write(sandbox.path().join("segments").join("stray.seg"), b"x")?;

    let error = observe(sandbox.path())
        .err()
        .ok_or("a stray pool entry was unexpectedly observed")?;

    let error = error
        .downcast::<GcLivenessObservationError>()
        .map_err(|_error| "observation refused with the wrong error type")?;
    assert!(matches!(*error, GcLivenessObservationError::PoolEntryName));
    sandbox.remove()?;
    Ok(())
}

/// Builds a canonical segment-retire receipt over the given coordinates for
/// the orphan pool segment.
pub(super) fn segment_receipt(
    coordinates: super::GcLivenessCoordinates,
    catalog_digest: crate::CatalogDigest,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use crate::adapters::{
        ArtifactIdentityDigest, CanonicalRecoveryDispositionReceipt, DecisionEvidenceDigest,
        ObservedHeadChecksum, ReaderLockDevice, ReaderLockFile, ReaderLockIdentity,
        ReaderLockMount, RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionArtifact,
        RecoveryDispositionCoordinates, RecoveryDispositionDecision, RecoveryDispositionReceipt,
    };
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    let identity = <[u8; 32]>::try_from(decode_hex(
        ORPHAN_SEGMENT_NAME
            .strip_suffix(".seg")
            .ok_or("orphan name")?,
    )?)
    .map_err(|_| "identity")?;
    let receipt = RecoveryDispositionReceipt::new(
        RecoveryDispositionArtifact {
            kind: RecoveryArtifactKind::Segment,
            classification: RecoveryClassification::CompleteOrphan,
            length: u64::try_from(orphan.len())?,
            identity_digest: ArtifactIdentityDigest::new(identity),
            content_digest: CanonicalRecoveryDispositionReceipt::artifact_content_digest(&orphan),
        },
        RecoveryDispositionDecision::Retire,
        RecoveryDispositionCoordinates {
            publication_generation: coordinates.catalog_generation(),
            publication_checksum: ObservedHeadChecksum::new([0; 32]),
            catalog_generation: coordinates.catalog_generation(),
            catalog_digest,
            retention: coordinates.retention(),
            reader_lock: ReaderLockIdentity::new(
                ReaderLockDevice::new(1),
                ReaderLockMount::new(2),
                ReaderLockFile::new(3),
            ),
        },
        DecisionEvidenceDigest::new([0; 32]),
    );
    Ok(CanonicalRecoveryDispositionReceipt::from_receipt(&receipt)
        .encoded()
        .to_vec())
}

pub(super) fn disposition_path(root: &Path) -> std::path::PathBuf {
    root.join("recovery").join("dispositions").join(format!(
        "{}.receipt",
        ORPHAN_SEGMENT_NAME.trim_end_matches(".seg")
    ))
}

#[test]
fn an_exact_retire_receipt_makes_the_orphan_collectible_and_a_stale_one_does_not()
-> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-disposed")?;
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;
    let coordinates = observe(sandbox.path())?.coordinates();

    // A receipt decided under another catalog digest is stale: protected.
    let stale = segment_receipt(
        coordinates,
        crate::CatalogDigest::from_validated([0x55; 32]),
    )?;
    fs::write(disposition_path(sandbox.path()), &stale)?;
    let plan = plan_gc(&observe(sandbox.path())?, GcLimits::MAXIMUM)?;
    assert_eq!(plan.candidate_count(), 0);
    assert_eq!(
        plan.segments()
            .values()
            .filter(|planned| {
                planned.classification() == GcSegmentClassification::RecoveryProtected
            })
            .count(),
        1
    );

    // The exact receipt releases it.
    let exact = segment_receipt(coordinates, coordinates.catalog_digest())?;
    fs::write(disposition_path(sandbox.path()), &exact)?;
    let plan = plan_gc(&observe(sandbox.path())?, GcLimits::MAXIMUM)?;
    assert_eq!(plan.candidate_count(), 1);
    let candidate = plan.candidates().next().ok_or("candidate")?;
    assert_eq!(
        plan.classification(candidate.segment()),
        Some(GcSegmentClassification::Unreachable(
            super::GcUnreachableEvidence::Disposed
        ))
    );

    // A receipt that does not decode refuses the whole observation.
    let mut corrupt = exact;
    let last = corrupt.last_mut().ok_or("receipt")?;
    *last ^= 1;
    fs::write(disposition_path(sandbox.path()), &corrupt)?;
    let error = observe(sandbox.path())
        .err()
        .ok_or("a corrupt disposition receipt was observed")?;
    let error = error
        .downcast::<GcLivenessObservationError>()
        .map_err(|_error| "observation refused with the wrong error type")?;
    assert!(matches!(
        *error,
        GcLivenessObservationError::Disposition { .. }
    ));
    sandbox.remove()?;
    Ok(())
}
