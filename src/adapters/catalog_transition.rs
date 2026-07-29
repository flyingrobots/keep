//! Exact generation and predecessor transition admission.

use super::{AdmittedCatalog, CatalogSuccessor, CatalogTransitionError};

pub(super) fn validate<'catalog, 'records>(
    current: &AdmittedCatalog<'_, '_>,
    candidate: AdmittedCatalog<'catalog, 'records>,
) -> Result<CatalogSuccessor<'catalog, 'records>, CatalogTransitionError> {
    let expected = current
        .generation()
        .successor()
        .map_err(|source| CatalogTransitionError::GenerationExhausted { source })?;
    let observed = candidate.generation();
    if observed != expected {
        return Err(CatalogTransitionError::Generation { expected, observed });
    }
    let expected = current.digest();
    let observed = candidate.previous_catalog_digest();
    if observed != Some(expected) {
        return Err(CatalogTransitionError::Predecessor { expected, observed });
    }
    Ok(CatalogSuccessor::new(candidate))
}
