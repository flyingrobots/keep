//! This module owns execution of the production GC retirement protocol.
//!
//! The child migrates the bundle store, publishes retention generation one,
//! adds the one-zero segment as an orphan pool entry with the exact retire
//! disposition that releases it, plans, and executes the retirement, dying
//! at the selected coordinate.

use std::fs;
use std::path::Path;

use keep::{
    ArtifactIdentityDigest, CanonicalRecoveryDispositionReceipt, DecisionEvidenceDigest,
    FilesystemGcAuthority, FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission, GcLimits,
    GcPlan, ObservedHeadChecksum, ReaderAttemptLimit, ReaderLockDevice, ReaderLockFile,
    ReaderLockIdentity, ReaderLockMount, RecoveryArtifactKind, RecoveryClassification,
    RecoveryDispositionArtifact, RecoveryDispositionCoordinates, RecoveryDispositionDecision,
    RecoveryDispositionReceipt, execute_gc, execute_retention_publication, observe_gc_liveness,
    plan_gc,
};

use super::control::CrashControl;
use super::fixture::{GoldenFixture, SEGMENT_POOL_PATH};
use super::gc_storage::CrashGcStorage;
use super::initialization;
use super::retention::{migrated_authority, preparation};
use super::{DurabilityCrashMatrixError, verification};

/// The orphan's pool-name digest, as lowercase hexadecimal.
const ORPHAN_DIGEST_HEX: &str = "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc";

pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    prepare_store(store_root)?;
    let plan = plan(store_root)?;
    let mut authority = gc_authority(store_root)?;
    let prepared = authority
        .prepare(&plan)
        .map_err(|source| verification("prepare production GC retirement", source))?;
    let mut storage = CrashGcStorage::new(authority, control, store_root);
    execute_gc(&mut storage, prepared.candidate_count())
        .map(|_receipt| ())
        .map_err(|source| verification("execute production GC retirement", source))
}

/// Migrates the bundle store, publishes retention generation one, and adds
/// the disposed orphan segment.
pub(in crate::durability_crash_matrix) fn prepare_store(
    store_root: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
    let mut authority = migrated_authority(store_root)?;
    let root = GoldenFixture::retention_root()?;
    let preparation = preparation(root.bytes())?;
    let _published = execute_retention_publication(&mut authority, &preparation)
        .map_err(|source| verification("publish crash GC retention generation", source))?;
    drop(authority);
    let orphan = GoldenFixture::segment()?;
    fs::write(store_root.join(SEGMENT_POOL_PATH), orphan.bytes())
        .map_err(|source| DurabilityCrashMatrixError::io("write crash GC orphan", source))?;
    let receipt = exact_receipt(store_root, orphan.bytes())?;
    fs::write(
        store_root
            .join("recovery")
            .join("dispositions")
            .join(format!("{ORPHAN_DIGEST_HEX}.receipt")),
        receipt,
    )
    .map_err(|source| DurabilityCrashMatrixError::io("write crash GC disposition", source))
}

/// Plans GC over the fenced view a production caller would take.
pub(in crate::durability_crash_matrix) fn plan(
    store_root: &Path,
) -> Result<GcPlan, DurabilityCrashMatrixError> {
    let view = FilesystemRetentionSnapshot::load(
        store_root,
        initialization::restart_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .map_err(|source| verification("load crash GC view", source))?;
    let snapshot = observe_gc_liveness(store_root, &view, initialization::restart_policy()?)
        .map_err(|source| verification("observe crash GC liveness", source))?;
    plan_gc(&snapshot, GcLimits::MAXIMUM)
        .map_err(|source| verification("plan crash GC retirement", source))
}

/// Reopens the store and returns GC authority over it.
pub(in crate::durability_crash_matrix) fn gc_authority(
    store_root: &Path,
) -> Result<FilesystemGcAuthority, DurabilityCrashMatrixError> {
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(store_root)
            .map_err(|source| verification("reopen crash store for GC", source))?;
    FilesystemGcAuthority::open(admission, store_root, initialization::restart_policy()?)
        .map_err(|source| verification("open crash GC authority", source))
}

/// The retire receipt for the orphan under exactly the store's current
/// planning coordinates, with fixture-only head-checksum, reader-lock, and
/// decision-evidence coordinates.
fn exact_receipt(store_root: &Path, orphan: &[u8]) -> Result<Vec<u8>, DurabilityCrashMatrixError> {
    let coordinates = plan(store_root)?.coordinates();
    let identity = decode_digest(ORPHAN_DIGEST_HEX)?;
    let receipt = RecoveryDispositionReceipt::new(
        RecoveryDispositionArtifact {
            kind: RecoveryArtifactKind::Segment,
            classification: RecoveryClassification::CompleteOrphan,
            length: u64::try_from(orphan.len())
                .map_err(|source| verification("convert crash GC orphan length", source))?,
            identity_digest: ArtifactIdentityDigest::new(identity),
            content_digest: CanonicalRecoveryDispositionReceipt::artifact_content_digest(orphan),
        },
        RecoveryDispositionDecision::Retire,
        RecoveryDispositionCoordinates {
            publication_generation: coordinates.catalog_generation(),
            publication_checksum: ObservedHeadChecksum::new([0; 32]),
            catalog_generation: coordinates.catalog_generation(),
            catalog_digest: coordinates.catalog_digest(),
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

fn decode_digest(hex: &str) -> Result<[u8; 32], DurabilityCrashMatrixError> {
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        let start = index.saturating_mul(2);
        let pair = hex
            .get(start..start.saturating_add(2))
            .ok_or(DurabilityCrashMatrixError::Usage)?;
        *byte =
            u8::from_str_radix(pair, 16).map_err(|_source| DurabilityCrashMatrixError::Usage)?;
    }
    Ok(digest)
}
