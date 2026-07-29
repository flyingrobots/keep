//! Consequential receipt for one fully synchronized catalog generation.

use crate::{CatalogDigest, CatalogGeneration};

/// Proof that publication reached root-directory synchronization.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogPublicationReceipt {
    generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
}

impl CatalogPublicationReceipt {
    /// Returns the exact published catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// Returns the exact published physical catalog digest.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    pub(super) const fn synchronized(
        generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
    ) -> Self {
        Self {
            generation,
            catalog_digest,
        }
    }
}
