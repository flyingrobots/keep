//! Consequential receipt for one synchronized catalog publication attempt.

use super::CatalogPublicationOutcome;
use crate::{CatalogDigest, CatalogGeneration};

/// Proof that a candidate became or remained current through root synchronization.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogPublicationReceipt {
    generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    outcome: CatalogPublicationOutcome,
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

    /// Returns whether this call published or reverified the generation.
    pub const fn outcome(self) -> CatalogPublicationOutcome {
        self.outcome
    }

    pub(super) const fn published(
        generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
    ) -> Self {
        Self {
            generation,
            catalog_digest,
            outcome: CatalogPublicationOutcome::Published,
        }
    }

    pub(super) const fn already_published(
        generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
    ) -> Self {
        Self {
            generation,
            catalog_digest,
            outcome: CatalogPublicationOutcome::AlreadyPublished,
        }
    }
}
