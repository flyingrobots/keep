//! This boundary module owns the coordinates one durable read binds.

use crate::adapters::{GcRetentionState, VerificationView};
use crate::{CatalogDigest, CatalogGeneration};

/// The exact view a durable snapshot pinned: the catalog generation and
/// digest `HEAD` selected and the retention state observed under the same
/// fence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurableView {
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    retention: GcRetentionState,
}

impl DurableView {
    pub(super) const fn new(
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

    /// The catalog generation the view selected.
    pub const fn catalog_generation(self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// That catalog's digest.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    /// The retention state observed under the same fence.
    #[must_use]
    pub const fn retention(self) -> GcRetentionState {
        self.retention
    }

    /// The same coordinates as a verification receipt's view.
    #[must_use]
    pub const fn verification_view(self) -> VerificationView {
        VerificationView::Durable {
            catalog_generation: self.catalog_generation,
            catalog_digest: self.catalog_digest,
            retention: self.retention,
        }
    }
}
