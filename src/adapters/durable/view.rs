//! This module owns the exact catalog and retention coordinates of a read.

use crate::adapters::{GcRetentionState, VerificationView};
use crate::{CatalogDigest, CatalogGeneration, RetentionHead};

/// Immutable coordinates admitted under one shared reader fence.
///
/// An absent retention head means no retention generation has been published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurableView {
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    retention: Option<RetentionHead>,
}

impl DurableView {
    pub(super) const fn new(
        catalog_generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
        retention: Option<RetentionHead>,
    ) -> Self {
        Self {
            catalog_generation,
            catalog_digest,
            retention,
        }
    }

    /// The admitted catalog generation.
    pub const fn catalog_generation(self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// The exact catalog digest selected by the admitted head.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    /// The complete retention head, including liveness generation and manifest digest.
    pub const fn retention(self) -> Option<RetentionHead> {
        self.retention
    }
    /// Returns the same catalog and liveness coordinates for a durable receipt.
    ///
    /// This projection does not grant retention authority or discard the complete
    /// head available through [`Self::retention`].
    #[must_use]
    pub const fn verification_view(self) -> VerificationView {
        let retention = match self.retention {
            None => GcRetentionState::Empty,
            Some(head) => GcRetentionState::Published {
                generation: head.generation(),
                manifest_digest: head.manifest_digest(),
            },
        };
        VerificationView::Durable {
            catalog_generation: self.catalog_generation,
            catalog_digest: self.catalog_digest,
            retention,
        }
    }
}
