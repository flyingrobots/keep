//! Exact successor catalog staged for publication.

use super::AdmittedCatalog;
use crate::CatalogGeneration;

/// Fully admitted catalog proven to be the exact successor of one snapshot.
#[must_use]
#[derive(Debug)]
pub struct CatalogSuccessor<'catalog, 'records> {
    catalog: AdmittedCatalog<'catalog, 'records>,
}

impl<'catalog, 'records> CatalogSuccessor<'catalog, 'records> {
    /// Returns the exact successor generation.
    pub const fn generation(&self) -> CatalogGeneration {
        self.catalog.generation()
    }

    /// Borrows the fully admitted successor catalog.
    pub const fn catalog(&self) -> &AdmittedCatalog<'catalog, 'records> {
        &self.catalog
    }

    /// Consumes the transition proof and returns the admitted catalog.
    pub fn into_catalog(self) -> AdmittedCatalog<'catalog, 'records> {
        self.catalog
    }

    pub(super) const fn new(catalog: AdmittedCatalog<'catalog, 'records>) -> Self {
        Self { catalog }
    }
}
