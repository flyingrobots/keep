//! This module owns admitted catalog coordinates for intent encoding.

use crate::{CatalogDigest, CatalogGeneration, CatalogLength, CatalogSnapshot};

#[derive(Clone, Copy)]
pub(super) struct MigrationCatalogCoordinates {
    generation: CatalogGeneration,
    length: CatalogLength,
    digest: CatalogDigest,
    predecessor: Option<CatalogDigest>,
}

impl MigrationCatalogCoordinates {
    pub(super) const fn new(
        generation: CatalogGeneration,
        length: CatalogLength,
        digest: CatalogDigest,
        predecessor: Option<CatalogDigest>,
    ) -> Self {
        Self {
            generation,
            length,
            digest,
            predecessor,
        }
    }

    pub(super) const fn from_snapshot(snapshot: &CatalogSnapshot<'_, '_, '_>) -> Self {
        Self::new(
            snapshot.generation(),
            snapshot.catalog_length(),
            snapshot.catalog_digest(),
            snapshot.previous_catalog_digest(),
        )
    }

    pub(super) const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    pub(super) const fn length(self) -> CatalogLength {
        self.length
    }

    pub(super) const fn digest(self) -> CatalogDigest {
        self.digest
    }

    pub(super) const fn predecessor(self) -> Option<CatalogDigest> {
        self.predecessor
    }
}
