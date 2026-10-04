//! This module owns bounded observations of catalog and retention heads.

use crate::{CatalogDigest, CatalogGeneration, CatalogLength, RetentionHead};

/// The coordinates both heads name at one instant.
///
/// A view is accepted only when the coordinates read before loading it equal
/// the coordinates read after, so the view belongs to one catalog generation
/// and one liveness generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionViewCoordinates {
    /// The catalog `HEAD` coordinate, or `None` when no catalog is published.
    pub catalog: Option<(CatalogGeneration, CatalogLength, CatalogDigest)>,
    /// The `retention/HEAD` coordinate, or `None` when no retention head is published.
    pub retention: Option<RetentionHead>,
}
