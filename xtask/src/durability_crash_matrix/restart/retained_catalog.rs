//! This module owns post-process-death verification of the writer.
//!
//! After the child dies at its coordinate, restart requires the exact stage
//! residue that coordinate leaves, recovers it with the public
//! recovery tools, republishes when the candidate never reached `HEAD`, and
//! then reads both the retained bundle blob and the ingested blob through
//! the production durable reader.

use std::fs;
use std::io;
use std::path::Path;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, BlobId, CanonicalCatalog,
    CanonicalRetentionRoot, CatalogGeneration, CatalogPublicationExpectation,
    CatalogPublicationOutcome, DurableStore, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemStoreAdmission, ReaderAttemptLimit, RecoveryEntryRole, RegisteredRetentionProfile,
    RetentionClosureLimits, RetentionGenerationExpectation, RetentionNamespace, RetentionPolicy,
    RetentionRoot, RootGeneration, SegmentPublication, execute_retention_publication,
    preflight_retention_transition, prepare_retention_publication,
    publish_retained_catalog_generation,
};
use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::crash_blob::{CrashBlob, SUCCESSOR_CONTENT};
use super::super::production_protocol::fixture::GoldenFixture;
use super::super::production_protocol::retained_catalog::{admit_segments, policy};
use super::super::production_protocol::retention::reopened_authority;
use super::super::production_protocol::verification;
use super::stage_recovery as recovery;

/// The catalog-protocol residue one coordinate leaves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Residue {
    /// A sealed `current.seg`, linked or not.
    SegmentStage,
    /// An empty `current.cat`.
    CatalogStage,
    /// A complete `head.next`.
    HeadStage,
    /// No stage; `HEAD` still names generation one.
    Unpublished,
    /// No stage; `HEAD` names the candidate.
    Published,
}

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let residue = residue(case)?;
    let observed = recovery::stage_roles(store_root)?;
    let expected: &[RecoveryEntryRole] = match residue {
        Residue::SegmentStage => &[RecoveryEntryRole::SegmentStage],
        Residue::CatalogStage => &[RecoveryEntryRole::CatalogStage],
        Residue::HeadStage => &[RecoveryEntryRole::NextHeadStage],
        Residue::Unpublished | Residue::Published => &[],
    };
    if observed != expected {
        return Err(mismatch(format!(
            "{} {:?}: expected stages {expected:?}, observed {observed:?}",
            case.point().identifier(),
            case.position()
        )));
    }
    recover(store_root, residue)?;
    require_generation_two(store_root)?;
    retain_blob(store_root)?;
    read_back(store_root)
}

fn residue(case: DurabilityCrashCase) -> Result<Residue, DurabilityCrashMatrixError> {
    let before = case.position() == DurabilityCrashPosition::Before;
    match case.point() {
        DurabilityCrashPoint::RetainedLinkSegment => Ok(Residue::SegmentStage),
        DurabilityCrashPoint::RetainedCreateCatalogStage if before => Ok(Residue::Unpublished),
        DurabilityCrashPoint::RetainedCreateCatalogStage => Ok(Residue::CatalogStage),
        DurabilityCrashPoint::RetainedSynchronizeHead => Ok(Residue::HeadStage),
        DurabilityCrashPoint::RetainedReplaceHead if before => Ok(Residue::HeadStage),
        DurabilityCrashPoint::RetainedReplaceHead => Ok(Residue::Published),
        point => Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    }
}

fn recover(store_root: &Path, residue: Residue) -> Result<(), DurabilityCrashMatrixError> {
    match residue {
        Residue::SegmentStage => {
            recovery::complete_segment(store_root)?;
            republish(store_root)
        }
        Residue::CatalogStage => {
            recovery::discard_catalog(store_root)?;
            republish(store_root)
        }
        Residue::Unpublished => republish(store_root),
        Residue::HeadStage => {
            let pool = pool_segments(store_root)?;
            let candidate: Vec<&[u8]> = pool.iter().map(Vec::as_slice).collect();
            recovery::finalize_head(store_root, &candidate)
        }
        Residue::Published => Ok(()),
    }
}

/// Publishes generation two over the pooled segments through the gate.
fn republish(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let pool = pool_segments(store_root)?;
    let encoded: Vec<&[u8]> = pool.iter().map(Vec::as_slice).collect();
    let segments = admit_segments(&encoded)?;
    let current = FilesystemCatalogSnapshot::load(store_root, policy()?)
        .map_err(|source| verification("load generation-one catalog", source))?;
    let expectation = CatalogPublicationExpectation::successor_of(
        &current
            .snapshot()
            .map_err(|source| verification("admit generation-one catalog", source))?,
    );
    let generation = CatalogGeneration::new(2)
        .map_err(|source| verification("construct republished generation", source))?;
    let catalog =
        CanonicalCatalog::from_segments(generation, Some(current.catalog_digest()), &segments)
            .map_err(|source| verification("encode republished catalog", source))?;
    let admission = FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(store_root)
        .map_err(|source| verification("reopen recovered store for publication", source))?;
    let mut publisher = FilesystemCatalogPublisher::open(admission, policy()?)
        .map_err(|source| verification("open recovered publisher", source))?;
    let receipt = publish_retained_catalog_generation(
        &mut publisher,
        expectation,
        SegmentPublication::none(),
        &catalog,
        &segments,
    )
    .map_err(|source| verification("republish recovered catalog", source))?;
    if receipt.outcome() == CatalogPublicationOutcome::Published {
        Ok(())
    } else {
        Err(mismatch(format!(
            "republication reported {:?}",
            receipt.outcome()
        )))
    }
}

fn require_generation_two(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let current = FilesystemCatalogSnapshot::load(store_root, policy()?)
        .map_err(|source| verification("load recovered catalog", source))?;
    if current.generation().get() == 2 {
        Ok(())
    } else {
        Err(mismatch(format!(
            "recovered HEAD names generation {}",
            current.generation().get()
        )))
    }
}

/// Retains the ingested blob in its own namespace, gated by the current catalog.
fn retain_blob(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let blob = CrashBlob::successor()?;
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(&b"version-two-crash"[..])
            .map_err(|source| verification("name crash retention namespace", source))?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 16_777_216, 67_108_864)
                .map_err(|source| verification("bound crash retention closure", source))?,
        ),
        None,
        vec![blob.anchor()],
    )
    .map_err(|source| verification("construct crash retention root", source))?;
    let canonical = CanonicalRetentionRoot::from_root(&root)
        .map_err(|source| verification("encode crash retention root", source))?;
    let candidate = AdmittedRetentionRoot::decode(canonical.encoded())
        .map_err(|source| verification("admit crash retention root", source))?;
    let current = FilesystemCatalogSnapshot::load(store_root, policy()?)
        .map_err(|source| verification("load recovered catalog for retention", source))?;
    let snapshot = current
        .snapshot()
        .map_err(|source| verification("admit recovered catalog for retention", source))?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )
    .map_err(|source| verification("preflight crash blob retention", source))?;
    let mut authority = reopened_authority(store_root)?;
    let observed = authority
        .observe_current()
        .map_err(|source| verification("observe recovered retention head", source))?;
    let manifest = observed
        .as_ref()
        .map(|state| AdmittedRetentionManifest::decode(state.manifest_bytes()))
        .transpose()
        .map_err(|source| verification("admit recovered retention manifest", source))?;
    let prepared = prepare_retention_publication(preflight, manifest.as_ref())
        .map_err(|source| verification("prepare crash blob retention", source))?;
    execute_retention_publication(&mut authority, &prepared)
        .map(|_receipt| ())
        .map_err(|source| verification("publish crash blob retention", source))
}

/// Reads the retained bundle blob and the ingested blob through the durable reader.
fn read_back(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let store = DurableStore::open(store_root, policy()?, ReaderAttemptLimit::DEFAULT)
        .map_err(|source| verification("open recovered durable store", source))?;
    let golden = GoldenFixture::retention_root()?;
    let retained = AdmittedRetentionRoot::decode(golden.bytes())
        .map_err(|source| verification("decode golden retention root", source))?;
    for anchor in retained.root().anchors() {
        let bytes = reconstruct(&store, anchor.blob_id())?;
        let observed = BlobId::hash_bytes(&bytes)
            .map_err(|source| verification("identify recovered retained blob", source))?;
        if observed != anchor.blob_id() {
            return Err(mismatch("recovered retained blob changed identity".into()));
        }
    }
    let ingested = reconstruct(&store, CrashBlob::successor()?.target())?;
    if ingested == SUCCESSOR_CONTENT {
        Ok(())
    } else {
        Err(mismatch(
            "recovered ingested blob differs from its content".into(),
        ))
    }
}

fn reconstruct(
    store: &DurableStore,
    target: BlobId,
) -> Result<Vec<u8>, DurabilityCrashMatrixError> {
    let mut output = Vec::new();
    store
        .reconstruct(target, &mut output)
        .map(|_receipt| output)
        .map_err(|source| verification("reconstruct recovered blob", source))
}

fn pool_segments(store_root: &Path) -> Result<Vec<Vec<u8>>, DurabilityCrashMatrixError> {
    let directory = store_root.join("segments");
    let mut names = fs::read_dir(&directory)
        .map_err(|source| DurabilityCrashMatrixError::io("list recovered segment pool", source))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| DurabilityCrashMatrixError::io("read recovered segment pool", source))?;
    names.sort();
    names
        .iter()
        .map(|name| {
            fs::read(directory.join(name))
                .map_err(|source| DurabilityCrashMatrixError::io("read recovered segment", source))
        })
        .collect()
}

fn mismatch(message: String) -> DurabilityCrashMatrixError {
    verification(
        "verify retained-catalog crash recovery",
        io::Error::other(message),
    )
}
