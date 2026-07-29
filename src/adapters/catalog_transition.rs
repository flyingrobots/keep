//! Exact generation and predecessor transition admission.

use super::{AdmittedCatalog, CatalogSuccessor, CatalogTransitionError};
use crate::{CatalogDigest, CatalogGeneration};

pub(super) fn validate<'catalog, 'records>(
    current: &AdmittedCatalog<'_, '_>,
    candidate: AdmittedCatalog<'catalog, 'records>,
) -> Result<CatalogSuccessor<'catalog, 'records>, CatalogTransitionError> {
    validate_coordinates(
        current.generation(),
        current.digest(),
        candidate.generation(),
        candidate.previous_catalog_digest(),
    )?;
    Ok(CatalogSuccessor::new(candidate))
}

pub(super) fn validate_coordinates(
    current_generation: CatalogGeneration,
    current_digest: CatalogDigest,
    candidate_generation: CatalogGeneration,
    candidate_previous_digest: Option<CatalogDigest>,
) -> Result<(), CatalogTransitionError> {
    let expected = current_generation
        .successor()
        .map_err(|source| CatalogTransitionError::GenerationExhausted { source })?;
    let observed = candidate_generation;
    if observed != expected {
        return Err(CatalogTransitionError::Generation { expected, observed });
    }
    let expected = current_digest;
    let observed = candidate_previous_digest;
    if observed != Some(expected) {
        return Err(CatalogTransitionError::Predecessor { expected, observed });
    }
    Ok(())
}
