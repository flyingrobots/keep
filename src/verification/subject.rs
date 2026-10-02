//! This module owns the coordinates naming a verification subject.

use crate::{CatalogDigest, CatalogGeneration};

/// Subject to which a verification claim or refusal applies.
///
/// Coordinates contain no paths or content bytes. A catalog coordinate is a
/// physical view selection, not a logical blob identity or retention promise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationSubject {
    /// One exact immutable catalog selected by a publication head.
    Catalog {
        /// Generation selected by the admitted head.
        generation: CatalogGeneration,
        /// Digest selected by that same admitted head.
        digest: CatalogDigest,
    },
}
