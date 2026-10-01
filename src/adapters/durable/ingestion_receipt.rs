//! This boundary module owns the durable ingestion receipt: what one
//! committed staging established, with exact byte accounting.

use crate::adapters::SegmentDigest;
use crate::{
    BlobId, CatalogDigest, CatalogGeneration, CommitReceipt, LayoutId, RegisteredStorageProfile,
};

/// Exact accounting of one staging: the logical length, the bytes written
/// into the new segment, and the bytes the pinned catalog already held.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IngestionAccounting {
    logical_bytes: u64,
    physical_new_bytes: u64,
    physical_reused_bytes: u64,
    chunks_new: u64,
    chunks_reused: u64,
}

impl IngestionAccounting {
    pub(super) const EMPTY: Self = Self {
        logical_bytes: 0,
        physical_new_bytes: 0,
        physical_reused_bytes: 0,
        chunks_new: 0,
        chunks_reused: 0,
    };

    pub(super) const fn new_chunk(&mut self, length: u64) {
        self.physical_new_bytes = self.physical_new_bytes.saturating_add(length);
        self.chunks_new = self.chunks_new.saturating_add(1);
    }

    pub(super) const fn reused_chunk(&mut self, length: u64) {
        self.physical_reused_bytes = self.physical_reused_bytes.saturating_add(length);
        self.chunks_reused = self.chunks_reused.saturating_add(1);
    }

    pub(super) const fn with_logical_bytes(mut self, logical_bytes: u64) -> Self {
        self.logical_bytes = logical_bytes;
        self
    }

    /// The source's exact length.
    #[must_use]
    pub const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }

    /// Chunk payload bytes written into the new segment.
    #[must_use]
    pub const fn physical_new_bytes(self) -> u64 {
        self.physical_new_bytes
    }

    /// Chunk payload bytes the pinned catalog already held, or this staging
    /// had already written.
    #[must_use]
    pub const fn physical_reused_bytes(self) -> u64 {
        self.physical_reused_bytes
    }

    /// Chunks written into the new segment.
    #[must_use]
    pub const fn chunks_new(self) -> u64 {
        self.chunks_new
    }

    /// Chunks reused by exact identity and representation.
    #[must_use]
    pub const fn chunks_reused(self) -> u64 {
        self.chunks_reused
    }
}

/// One committed durable staging.
///
/// `segment` is `None` when the pinned catalog already held every chunk and
/// the layout: nothing was published and `generation` is the one the
/// staging was verified against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the receipt records the committed identities, coordinates, and accounting"]
pub struct DurableIngestionReceipt {
    target: BlobId,
    layout_id: LayoutId,
    profile: RegisteredStorageProfile,
    segment: Option<SegmentDigest>,
    generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    accounting: IngestionAccounting,
}

impl DurableIngestionReceipt {
    pub(super) const fn new(
        target: BlobId,
        layout_id: LayoutId,
        segment: Option<SegmentDigest>,
        catalog: (CatalogGeneration, CatalogDigest),
        accounting: IngestionAccounting,
    ) -> Self {
        Self {
            target,
            layout_id,
            profile: RegisteredStorageProfile::FAST_CDC_64K_V1,
            segment,
            generation: catalog.0,
            catalog_digest: catalog.1,
            accounting,
        }
    }

    /// The committed blob.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// The exact committed layout.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }

    /// The storage profile the layout was derived under.
    #[must_use]
    pub const fn profile(self) -> RegisteredStorageProfile {
        self.profile
    }

    /// The published segment, when anything was new.
    #[must_use]
    pub const fn segment(self) -> Option<SegmentDigest> {
        self.segment
    }

    /// The catalog generation the content is visible under.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// That generation's catalog digest.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    /// The exact byte accounting.
    #[must_use]
    pub const fn accounting(self) -> IngestionAccounting {
        self.accounting
    }
}

impl CommitReceipt for DurableIngestionReceipt {
    fn target(&self) -> BlobId {
        self.target
    }

    fn layout_id(&self) -> LayoutId {
        self.layout_id
    }
}
