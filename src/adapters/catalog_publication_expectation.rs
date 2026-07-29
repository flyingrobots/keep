//! Typed current-state expectation for catalog publication.

use super::CatalogSnapshot;
use crate::{CatalogDigest, CatalogGeneration};

/// Current durable state that a writer must revalidate before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogPublicationExpectation {
    current: ExpectedCurrentCatalog,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExpectedCurrentCatalog {
    Uninitialized,
    Published {
        generation: CatalogGeneration,
        digest: CatalogDigest,
    },
}

impl CatalogPublicationExpectation {
    /// Expects a store with no admitted publication head.
    pub const fn uninitialized() -> Self {
        Self {
            current: ExpectedCurrentCatalog::Uninitialized,
        }
    }

    /// Expects the exact current coordinates pinned by `snapshot`.
    pub const fn successor_of(snapshot: &CatalogSnapshot<'_, '_, '_>) -> Self {
        Self {
            current: ExpectedCurrentCatalog::Published {
                generation: snapshot.generation(),
                digest: snapshot.catalog_digest(),
            },
        }
    }

    /// Returns the expected current generation, absent for an uninitialized store.
    #[must_use]
    pub const fn current_generation(self) -> Option<CatalogGeneration> {
        match self.current {
            ExpectedCurrentCatalog::Uninitialized => None,
            ExpectedCurrentCatalog::Published { generation, .. } => Some(generation),
        }
    }

    /// Returns the expected current digest, absent for an uninitialized store.
    #[must_use]
    pub const fn current_catalog_digest(self) -> Option<CatalogDigest> {
        match self.current {
            ExpectedCurrentCatalog::Uninitialized => None,
            ExpectedCurrentCatalog::Published { digest, .. } => Some(digest),
        }
    }

    pub(super) const fn current(self) -> ExpectedCurrentCatalog {
        self.current
    }
}
