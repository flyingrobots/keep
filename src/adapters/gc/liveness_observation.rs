//! This boundary module assembles one liveness snapshot from a fenced view.
//!
//! Observation reads; it never writes. It re-admits the fenced catalog,
//! projects every retained root's verified closure onto segments, reads and
//! admits the whole segment pool, walks the catalog predecessor chain for
//! superseded segments, and hands the result to the pure planner. No
//! recovery-disposition receipt exists yet, so nothing is reported disposed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::{
    AdmittedRecoveryDispositionReceipt, RecoveryArtifactKind, RecoveryDispositionDecision,
    VerificationEvidenceDigest,
};
use super::{
    GcLivenessCoordinates, GcLivenessObservationError as Error, GcLivenessSnapshot,
    GcRetainedClosure, GcRetentionState,
};
use crate::adapters::retention::{
    AdmittedRetentionRoot, RetentionClosureMembers, is_disposition_name,
    verify_retention_closure_members,
};
use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartPhase, CatalogRestartPolicy, CatalogSnapshot,
    ChecksummedCatalog, FilesystemRetentionSnapshot, SegmentDigest, SegmentRecordIdentity,
    catalog_restart_io, physical_pool_name,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

use super::segment_pool_inventory;

const SEGMENTS: &str = "segments";
const CATALOGS: &str = "catalogs";
const RECOVERY: &str = "recovery";
const DISPOSITIONS: &str = "dispositions";
const RECEIPT_LENGTH: usize = 320;

/// Assembles the liveness snapshot the fenced `view` of `store_root` admits.
///
/// `policy` bounds the segment grammar and the total bytes read from the
/// segment pool. The returned snapshot binds the view's catalog generation
/// and digest and its retention state, so a plan computed from it can only
/// be executed against exactly that view.
///
/// # Errors
///
/// Returns [`GcLivenessObservationError`](Error) at the exact catalog, root,
/// closure, pool, or chain refusal. No partial snapshot is returned.
pub fn observe_gc_liveness(
    store_root: &Path,
    view: &FilesystemRetentionSnapshot,
    policy: CatalogRestartPolicy,
) -> Result<GcLivenessSnapshot, Error> {
    crate::adapters::filesystem_root_binding::require_locator(view.root_directory(), store_root)
        .map_err(|source| Error::pool("bind store root", source))?;
    observe_gc_liveness_from_view(view, policy)
}

pub(in crate::adapters) fn observe_gc_liveness_from_view(
    view: &FilesystemRetentionSnapshot,
    policy: CatalogRestartPolicy,
) -> Result<GcLivenessSnapshot, Error> {
    let catalog = view
        .catalog()
        .snapshot()
        .map_err(|source| Error::Catalog { source })?;
    let record_segments = catalog
        .record_segments()
        .map_err(|source| Error::CatalogEntries { source })?;
    let mut snapshot = GcLivenessSnapshot::new(GcLivenessCoordinates::new(
        catalog.generation(),
        catalog.catalog_digest(),
        retention_state(view),
    ));
    let named: BTreeSet<SegmentDigest> = record_segments.values().copied().collect();
    for segment in &named {
        snapshot.name_segment(*segment);
    }
    retain_closures(view, &catalog, &record_segments, &mut snapshot)?;
    let root = view.root_directory();
    let segments = root
        .open_dir_nofollow(SEGMENTS)
        .map_err(|source| Error::pool("open segment pool", source))?;
    let inventory = segment_pool_inventory::read(
        &segments,
        policy.segment_read(),
        policy.retained_segment_bytes().get(),
    )?;
    for (segment, length) in inventory {
        snapshot
            .inventory_segment(segment, length)
            .map_err(|source| Error::Snapshot { source })?;
    }
    let catalogs = root
        .open_dir_nofollow(CATALOGS)
        .map_err(|source| Error::pool("open catalog pool", source))?;
    for (segment, evidence) in superseded_segments(&catalogs, &catalog, &named)? {
        snapshot.supersede_segment(segment, evidence);
    }
    let recovery = root
        .open_dir_nofollow(RECOVERY)
        .map_err(|source| Error::pool("open recovery directory", source))?;
    let dispositions = recovery
        .open_dir_nofollow(DISPOSITIONS)
        .map_err(|source| Error::pool("open disposition pool", source))?;
    for (segment, evidence) in disposed_segments(&dispositions, snapshot.coordinates())? {
        snapshot.dispose_segment(segment, evidence);
    }
    Ok(snapshot)
}

/// Reads every disposition receipt and admits only the exact ones: a
/// `segment` artifact, a `retire` decision, an entry named by its own
/// identity digest, and coordinates equal to this snapshot's. A receipt
/// decided under other coordinates is stale and keeps its segment
/// protected; a receipt that does not decode refuses the observation.
fn disposed_segments(
    dispositions: &Dir,
    coordinates: GcLivenessCoordinates,
) -> Result<BTreeMap<SegmentDigest, VerificationEvidenceDigest>, Error> {
    let mut disposed = BTreeMap::new();
    for entry in dispositions
        .entries()
        .map_err(|source| Error::pool("list dispositions", source))?
    {
        let entry = entry.map_err(|source| Error::pool("read disposition entry", source))?;
        let name = entry.file_name();
        if !is_disposition_name(&name) {
            return Err(Error::DispositionEntryName);
        }
        let name = name.to_string_lossy().into_owned();
        let bytes = catalog_restart_io::open_regular(
            dispositions,
            &name,
            CatalogRestartArtifact::Catalog,
            CatalogRestartPhase::OpenCatalog,
        )
        .and_then(|(file, length)| {
            catalog_restart_io::read_exact(
                file,
                CatalogRestartArtifact::Catalog,
                CatalogRestartPhase::ReadCatalog,
                length.min(
                    u64::try_from(RECEIPT_LENGTH)
                        .unwrap_or(u64::MAX)
                        .saturating_add(1),
                ),
            )
        })
        .map_err(|source| Error::Catalog { source })?;
        let admitted = AdmittedRecoveryDispositionReceipt::decode(&bytes)
            .map_err(|source| Error::Disposition { source })?;
        let receipt = admitted.receipt();
        let artifact = receipt.artifact();
        let identity = SegmentDigest::from_validated(*artifact.identity_digest.as_bytes());
        if name != physical_pool_name::disposition(artifact.identity_digest.as_bytes()) {
            return Err(Error::DispositionEntryName);
        }
        let exact = artifact.kind == RecoveryArtifactKind::Segment
            && receipt.decision() == RecoveryDispositionDecision::Retire
            && receipt.coordinates().catalog_generation == coordinates.catalog_generation()
            && receipt.coordinates().catalog_digest == coordinates.catalog_digest()
            && receipt.coordinates().retention == coordinates.retention();
        if exact {
            let checksum: [u8; 32] = bytes
                .get(RECEIPT_LENGTH.saturating_sub(32)..RECEIPT_LENGTH)
                .and_then(|slice| slice.try_into().ok())
                .ok_or(Error::DispositionEntryName)?;
            disposed.insert(identity, VerificationEvidenceDigest::new(checksum));
        }
    }
    Ok(disposed)
}

fn retention_state(view: &FilesystemRetentionSnapshot) -> GcRetentionState {
    view.retention_head()
        .map_or(GcRetentionState::Empty, |head| {
            GcRetentionState::Published {
                generation: head.generation(),
                manifest_digest: head.manifest_digest(),
            }
        })
}

/// Decodes and verifies every retained root's closure against `catalog`,
/// handing each to `visit` with its resolved member identities.
///
/// # Errors
///
/// Returns the exact retained-root, closure, or visitor refusal.
pub(in crate::adapters) fn visit_retained_closures(
    view: &FilesystemRetentionSnapshot,
    catalog: &CatalogSnapshot<'_, '_, '_>,
    mut visit: impl FnMut(
        crate::RetentionNamespaceDigest,
        &AdmittedRetentionRoot<'_>,
        &RetentionClosureMembers,
    ) -> Result<(), Error>,
) -> Result<(), Error> {
    let Some(manifest) = view.manifest() else {
        return Ok(());
    };
    for entry in manifest.entries() {
        let namespace = entry.namespace();
        let bytes = view
            .retained_root(namespace)
            .map_err(|source| Error::RetainedRoot {
                namespace,
                source: Box::new(source),
            })?
            .ok_or(Error::RetainedRootAbsent { namespace })?;
        let root = AdmittedRetentionRoot::decode(&bytes).map_err(|source| Error::RetainedRoot {
            namespace,
            source: Box::new(source),
        })?;
        let members = verify_retention_closure_members(root.root(), catalog).map_err(|source| {
            Error::Closure {
                namespace,
                source: Box::new(source),
            }
        })?;
        visit(namespace, &root, &members)?;
    }
    Ok(())
}

fn retain_closures(
    view: &FilesystemRetentionSnapshot,
    catalog: &CatalogSnapshot<'_, '_, '_>,
    record_segments: &BTreeMap<SegmentRecordIdentity, SegmentDigest>,
    snapshot: &mut GcLivenessSnapshot,
) -> Result<(), Error> {
    visit_retained_closures(view, catalog, |namespace, root, members| {
        let mut segments = BTreeSet::new();
        for identity in &members.identities {
            let segment = record_segments
                .get(identity)
                .ok_or(Error::ClosureMemberUnindexed { namespace })?;
            segments.insert(*segment);
        }
        snapshot
            .retain(GcRetainedClosure::new(
                namespace,
                root.root().generation(),
                root.digest(),
                members.closure.digest(),
                segments,
            ))
            .map_err(|source| Error::Snapshot { source })
    })
}

/// Walks the catalog chain from the current catalog's predecessor to
/// generation one and returns every segment a predecessor names that the
/// current catalog does not.
fn superseded_segments(
    catalogs: &Dir,
    current: &CatalogSnapshot<'_, '_, '_>,
    named: &BTreeSet<SegmentDigest>,
) -> Result<BTreeMap<SegmentDigest, VerificationEvidenceDigest>, Error> {
    let mut superseded = BTreeMap::new();
    let mut generation = current.generation();
    let mut next = current.previous_catalog_digest();
    while let Some(digest) = next {
        generation = predecessor_generation(generation)?;
        let bytes = read_catalog(catalogs, generation, digest)?;
        let catalog =
            ChecksummedCatalog::decode(&bytes).map_err(|source| Error::PredecessorCatalog {
                generation,
                source: Box::new(source),
            })?;
        if catalog.generation() != generation || catalog.digest() != digest {
            return Err(Error::PredecessorCatalogMismatch { generation });
        }
        let segments = catalog
            .segment_digests()
            .map_err(|source| Error::PredecessorCatalog {
                generation,
                source: Box::new(source),
            })?;
        for segment in segments.difference(named) {
            superseded
                .entry(*segment)
                .or_insert_with(|| VerificationEvidenceDigest::new(*digest.as_bytes()));
        }
        next = catalog.previous_catalog_digest();
    }
    Ok(superseded)
}

fn predecessor_generation(generation: CatalogGeneration) -> Result<CatalogGeneration, Error> {
    let previous = generation
        .get()
        .checked_sub(1)
        .ok_or(Error::PredecessorCatalogMismatch { generation })?;
    CatalogGeneration::new(previous)
        .map_err(|_source| Error::PredecessorCatalogMismatch { generation })
}

fn read_catalog(
    catalogs: &Dir,
    generation: CatalogGeneration,
    digest: CatalogDigest,
) -> Result<Vec<u8>, Error> {
    let name = physical_pool_name::catalog(generation, digest);
    let restart = |source| Error::PredecessorCatalog {
        generation,
        source: Box::new(source),
    };
    let (file, length) = catalog_restart_io::open_regular(
        catalogs,
        &name,
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::OpenCatalog,
    )
    .map_err(restart)?;
    if length > CatalogLength::MAXIMUM.get() {
        return Err(Error::PredecessorCatalogMismatch { generation });
    }
    catalog_restart_io::read_exact(
        file,
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::ReadCatalog,
        length,
    )
    .map_err(restart)
}
