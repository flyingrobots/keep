//! This module owns validated catalog coordinates for next-head finalization.

use super::CatalogSnapshot;
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Exact catalog coordinate named by a complete recovery `head.next`.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryNextHeadFinalizationTarget {
    generation: CatalogGeneration,
    length: CatalogLength,
    digest: CatalogDigest,
}

impl RecoveryNextHeadFinalizationTarget {
    /// Derives exact coordinates from a complete pinned catalog snapshot.
    pub const fn from_snapshot(snapshot: &CatalogSnapshot<'_, '_, '_>) -> Self {
        Self::new(
            snapshot.generation(),
            snapshot.catalog_length(),
            snapshot.catalog_digest(),
        )
    }

    pub(super) const fn new(
        generation: CatalogGeneration,
        length: CatalogLength,
        digest: CatalogDigest,
    ) -> Self {
        Self {
            generation,
            length,
            digest,
        }
    }

    /// Returns the exact candidate generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// Returns the exact candidate catalog byte length.
    pub const fn length(self) -> CatalogLength {
        self.length
    }

    /// Returns the exact candidate catalog digest.
    pub const fn digest(self) -> CatalogDigest {
        self.digest
    }
}
