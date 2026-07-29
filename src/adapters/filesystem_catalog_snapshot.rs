//! This module owns an immutable, restart-loaded filesystem snapshot.

use std::path::Path;

use super::loaded_segment::LoadedSegment;
use super::{
    AdmittedSegment, CatalogRestartError, CatalogRestartPolicy, CatalogSnapshot,
    ChecksummedCatalog, ChecksummedPublicationHead, catalog_restart_loader,
};
use crate::{CatalogDigest, CatalogGeneration};

/// Owned bytes and proofs for one exact head-selected catalog generation.
///
/// Loading follows only the exact catalog and segment coordinates named by the
/// checksummed publication state. Unknown files and orphaned artifacts are not
/// recovery candidates and are deliberately ignored.
#[must_use]
pub struct FilesystemCatalogSnapshot {
    head_bytes: Vec<u8>,
    catalog_bytes: Vec<u8>,
    segments: Vec<LoadedSegment>,
    policy: CatalogRestartPolicy,
    generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
}

impl FilesystemCatalogSnapshot {
    /// Loads and admits the exact snapshot selected by `HEAD`.
    ///
    /// The returned owner retains bounded segment bytes so every later logical
    /// lookup remains pinned to this immutable generation.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogRestartError`] for filesystem refusal, malformed or
    /// noncanonical bytes, coordinate disagreement, resource-limit refusal, or
    /// failed catalog-to-record admission.
    pub fn load(root: &Path, policy: CatalogRestartPolicy) -> Result<Self, CatalogRestartError> {
        catalog_restart_loader::load(root, policy)
    }

    /// Returns the exact generation selected during restart.
    pub const fn generation(&self) -> CatalogGeneration {
        self.generation
    }

    /// Returns the verified digest of the selected canonical catalog.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.catalog_digest
    }

    /// Reconstructs a borrowed logical snapshot from retained immutable bytes.
    ///
    /// Admission is repeated so no unchecked or serializer-owned state is
    /// retained between the physical bytes and the logical reader view.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogRestartError`] if retained bytes fail any decoder,
    /// physical-coordinate, record-binding, or head-binding invariant.
    pub fn snapshot(&self) -> Result<CatalogSnapshot<'_, '_, '_>, CatalogRestartError> {
        let head = ChecksummedPublicationHead::decode(&self.head_bytes)
            .map_err(|source| CatalogRestartError::Head { source })?;
        let catalog = ChecksummedCatalog::decode(&self.catalog_bytes)
            .map_err(|source| CatalogRestartError::Catalog { source })?;
        let segment_count = u64::try_from(self.segments.len())
            .map_err(|_source| CatalogRestartError::SegmentIndexLength)?;
        let mut segments = Vec::new();
        segments
            .try_reserve_exact(self.segments.len())
            .map_err(|source| CatalogRestartError::SegmentIndexAllocation {
                segment_count,
                source,
            })?;
        for loaded in &self.segments {
            let segment = AdmittedSegment::decode(loaded.encoded(), self.policy.segment_read())
                .map_err(|source| CatalogRestartError::Segment {
                    expected: loaded.digest(),
                    source: Box::new(source),
                })?;
            if segment.digest() != loaded.digest() {
                return Err(CatalogRestartError::SegmentCoordinate {
                    expected: loaded.digest(),
                    observed: segment.digest(),
                });
            }
            segments.push(segment);
        }
        let catalog =
            catalog
                .admit(&segments)
                .map_err(|source| CatalogRestartError::CatalogAdmission {
                    source: Box::new(source),
                })?;
        head.admit(catalog)
            .map_err(|source| CatalogRestartError::Snapshot { source })
    }

    pub(super) fn admit(
        head_bytes: Vec<u8>,
        catalog_bytes: Vec<u8>,
        segments: Vec<LoadedSegment>,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, CatalogRestartError> {
        let head = ChecksummedPublicationHead::decode(&head_bytes)
            .map_err(|source| CatalogRestartError::Head { source })?;
        let generation = head.generation();
        let catalog_digest = head.catalog_digest();
        let snapshot = Self {
            head_bytes,
            catalog_bytes,
            segments,
            policy,
            generation,
            catalog_digest,
        };
        {
            let _validated = snapshot.snapshot()?;
        }
        Ok(snapshot)
    }

    pub(super) fn head_bytes(&self) -> &[u8] {
        &self.head_bytes
    }
}
