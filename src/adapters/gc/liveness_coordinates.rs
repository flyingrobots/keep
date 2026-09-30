//! This boundary module owns the immutable coordinates a GC plan binds.

use crate::{CatalogDigest, CatalogGeneration, LivenessGeneration, RetentionManifestDigest};

/// The retention state a liveness snapshot was taken under.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcRetentionState {
    /// No retention head has been published: the canonical empty retention
    /// state, in which nothing is retained and every anchor-less segment is
    /// unreachable from retention.
    Empty,
    /// One published retention head selects one manifest.
    Published {
        /// The manifest's liveness generation.
        generation: LivenessGeneration,
        /// The exact manifest digest.
        manifest_digest: RetentionManifestDigest,
    },
}

/// The exact catalog and retention view one GC plan was computed against.
///
/// A plan is a statement about exactly these coordinates. Execution must
/// reopen the store and prove the same coordinates before acting on it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcLivenessCoordinates {
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    retention: GcRetentionState,
}

impl GcLivenessCoordinates {
    /// Binds one catalog generation and digest to one retention state.
    #[must_use]
    pub const fn new(
        catalog_generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
        retention: GcRetentionState,
    ) -> Self {
        Self {
            catalog_generation,
            catalog_digest,
            retention,
        }
    }

    /// Returns the catalog generation records were resolved against.
    pub const fn catalog_generation(self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// Returns the exact catalog digest records were resolved against.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    /// Returns the retention state the snapshot was taken under.
    #[must_use]
    pub const fn retention(self) -> GcRetentionState {
        self.retention
    }
}
