//! Publication-head to catalog-snapshot admission failures.

use std::error::Error;
use std::fmt;

use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Failure to bind one checksummed head to one fully admitted catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogSnapshotError {
    /// Head and catalog generations disagreed.
    Generation {
        /// Generation named by the head.
        expected: CatalogGeneration,
        /// Generation verified from the catalog.
        observed: CatalogGeneration,
    },
    /// Head and catalog byte lengths disagreed.
    CatalogLength {
        /// Catalog length named by the head.
        expected: CatalogLength,
        /// Verified catalog length.
        observed: CatalogLength,
    },
    /// Head and catalog physical digests disagreed.
    CatalogDigest {
        /// Catalog digest named by the head.
        expected: CatalogDigest,
        /// Verified catalog digest.
        observed: CatalogDigest,
    },
}

impl fmt::Display for CatalogSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generation { expected, observed } => write!(
                formatter,
                "head generation {} disagrees with catalog generation {}",
                expected.get(),
                observed.get()
            ),
            Self::CatalogLength { expected, observed } => write!(
                formatter,
                "head catalog length {} disagrees with verified length {}",
                expected.get(),
                observed.get()
            ),
            Self::CatalogDigest { .. } => formatter.write_str("head and catalog digests disagree"),
        }
    }
}

impl Error for CatalogSnapshotError {}
