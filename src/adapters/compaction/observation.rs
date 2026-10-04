//! This boundary module assembles what compaction plans over: every record
//! the current catalog names with its segment, every record some retained
//! closure reaches, and the physical segment inventory.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cap_fs_ext::DirExt;

use crate::adapters::gc::{
    GcLivenessCoordinates, GcLivenessObservationError, GcRetentionState, visit_retained_closures,
};
use crate::adapters::{
    CatalogRestartPolicy, FilesystemRetentionSnapshot, SegmentDigest, SegmentRecordIdentity,
};

/// Everything the compaction planner reads, observed under one view.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionObservation {
    coordinates: GcLivenessCoordinates,
    named: BTreeMap<SegmentRecordIdentity, SegmentDigest>,
    live: BTreeSet<SegmentRecordIdentity>,
    inventory: BTreeMap<SegmentDigest, u64>,
}

impl CompactionObservation {
    /// Builds an observation from its parts; the planner is pure over it.
    pub const fn new(
        coordinates: GcLivenessCoordinates,
        named: BTreeMap<SegmentRecordIdentity, SegmentDigest>,
        live: BTreeSet<SegmentRecordIdentity>,
        inventory: BTreeMap<SegmentDigest, u64>,
    ) -> Self {
        Self {
            coordinates,
            named,
            live,
            inventory,
        }
    }

    /// The catalog and retention coordinates the observation binds.
    #[must_use]
    pub const fn coordinates(&self) -> GcLivenessCoordinates {
        self.coordinates
    }

    /// Every record the current catalog names, with the segment holding it.
    #[must_use]
    pub const fn named(&self) -> &BTreeMap<SegmentRecordIdentity, SegmentDigest> {
        &self.named
    }

    /// Every record some retained closure reaches.
    #[must_use]
    pub const fn live(&self) -> &BTreeSet<SegmentRecordIdentity> {
        &self.live
    }

    /// Every admitted segment-pool entry with its length.
    #[must_use]
    pub const fn inventory(&self) -> &BTreeMap<SegmentDigest, u64> {
        &self.inventory
    }
}

/// Observes compaction inputs from one fenced (or writer-authority) view.
///
/// # Errors
///
/// Returns [`GcLivenessObservationError`] at the exact catalog, closure,
/// or pool refusal; nothing is planned from a partially admitted store.
pub fn observe_compaction(
    store_root: &Path,
    view: &FilesystemRetentionSnapshot,
    policy: CatalogRestartPolicy,
) -> Result<CompactionObservation, GcLivenessObservationError> {
    crate::adapters::filesystem_root_binding::require_locator(view.root_directory(), store_root)
        .map_err(|source| GcLivenessObservationError::pool("bind store root", source))?;
    observe_compaction_from_view(view, policy)
}

pub(super) fn observe_compaction_from_view(
    view: &FilesystemRetentionSnapshot,
    policy: CatalogRestartPolicy,
) -> Result<CompactionObservation, GcLivenessObservationError> {
    let catalog = view
        .catalog()
        .snapshot()
        .map_err(|source| GcLivenessObservationError::Catalog { source })?;
    let named = catalog
        .record_segments()
        .map_err(|source| GcLivenessObservationError::CatalogEntries { source })?;
    let mut live = BTreeSet::new();
    visit_retained_closures(view, &catalog, |_namespace, _root, members| {
        live.extend(members.identities.iter().copied());
        Ok(())
    })?;
    let root = view.root_directory();
    let segments = root
        .open_dir_nofollow("segments")
        .map_err(|source| GcLivenessObservationError::pool("open segment pool", source))?;
    let inventory = crate::adapters::gc::read_segment_pool_inventory(
        &segments,
        policy.segment_read(),
        policy.retained_segment_bytes().get(),
    )?
    .into_iter()
    .collect();
    let retention = view
        .retention_head()
        .map_or(GcRetentionState::Empty, |head| {
            GcRetentionState::Published {
                generation: head.generation(),
                manifest_digest: head.manifest_digest(),
            }
        });
    Ok(CompactionObservation::new(
        GcLivenessCoordinates::new(catalog.generation(), catalog.catalog_digest(), retention),
        named,
        live,
        inventory,
    ))
}
