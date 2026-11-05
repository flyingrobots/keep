//! This module owns the state a caller expects before trusting an opened store.

use std::error::Error;
use std::fmt;

use crate::{CatalogDigest, CatalogGeneration};

/// The exact published catalog a caller recorded outside the store.
///
/// A publication receipt or a pinned view supplies both values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedCatalog {
    generation: CatalogGeneration,
    digest: CatalogDigest,
}

impl ExpectedCatalog {
    /// Names one exact catalog generation and digest.
    #[must_use]
    pub const fn new(generation: CatalogGeneration, digest: CatalogDigest) -> Self {
        Self { generation, digest }
    }

    /// The expected catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// The expected catalog digest.
    pub const fn digest(self) -> CatalogDigest {
        self.digest
    }
}

/// What a caller knows about the store it is opening.
///
/// A store's own records cannot show that it is the store the caller meant, or
/// that newer history existed, because a restored store carries consistent old
/// records. Only state kept outside the store can. Every open therefore requires
/// this choice, and it has no default, so no caller loses the check by omission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreExpectation {
    /// Refuse unless the store's published catalog is exactly this one.
    ///
    /// This does not detect a rollback that leaves the catalog unchanged, such as
    /// restoring only older retention state.
    Catalog(ExpectedCatalog),
    /// The caller has no expectation and accepts any valid store at this path,
    /// including a different store or an older state of the intended one.
    NoExpectation,
}

impl StoreExpectation {
    /// Compares the observed published catalog, `None` meaning none is published.
    pub(super) fn require(
        self,
        observed: Option<ExpectedCatalog>,
    ) -> Result<(), StoreExpectationMismatch> {
        match self {
            Self::Catalog(expected) if observed != Some(expected) => {
                Err(StoreExpectationMismatch { expected, observed })
            }
            Self::Catalog(_) | Self::NoExpectation => Ok(()),
        }
    }
}

/// An opened store does not hold the catalog its caller expected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreExpectationMismatch {
    expected: ExpectedCatalog,
    observed: Option<ExpectedCatalog>,
}

impl StoreExpectationMismatch {
    /// The catalog the caller expected.
    #[must_use]
    pub const fn expected(self) -> ExpectedCatalog {
        self.expected
    }

    /// The catalog the store has published, or `None` when it has published none.
    #[must_use]
    pub const fn observed(self) -> Option<ExpectedCatalog> {
        self.observed
    }
}

impl fmt::Display for StoreExpectationMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the store does not hold the expected published catalog")
    }
}

impl Error for StoreExpectationMismatch {}
