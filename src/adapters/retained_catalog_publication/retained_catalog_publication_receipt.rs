//! This module owns the receipt for one retained-closure catalog publication.

use super::super::{CatalogPublicationOutcome, CatalogPublicationReceipt};
use super::PreservedRetentionManifest;
use crate::{CatalogDigest, CatalogGeneration};

/// Proof that a candidate became or remained current while preserving retention.
///
/// The receipt binds the catalog coordinates with the exact retention manifest
/// whose every root closure the candidate was verified to preserve, and whose
/// head was unchanged immediately before `HEAD` replacement.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetainedCatalogPublicationReceipt {
    catalog: CatalogPublicationReceipt,
    preserved: Option<PreservedRetentionManifest>,
}

impl RetainedCatalogPublicationReceipt {
    /// Returns the underlying catalog publication receipt.
    pub const fn catalog(self) -> CatalogPublicationReceipt {
        self.catalog
    }

    /// Returns the exact published catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.catalog.generation()
    }

    /// Returns the exact published physical catalog digest.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog.catalog_digest()
    }

    /// Returns whether this call published or reverified the generation.
    pub const fn outcome(self) -> CatalogPublicationOutcome {
        self.catalog.outcome()
    }

    /// Returns the preserved retention manifest.
    ///
    /// `None` means no retention head was published when the gate admitted
    /// evidence, so there were no retained closures to preserve.
    #[must_use]
    pub const fn preserved_manifest(self) -> Option<PreservedRetentionManifest> {
        self.preserved
    }

    pub(super) const fn new(
        catalog: CatalogPublicationReceipt,
        preserved: Option<PreservedRetentionManifest>,
    ) -> Self {
        Self { catalog, preserved }
    }
}
