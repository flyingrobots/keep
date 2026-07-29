//! Exact catalog-successor transition failures.

use std::error::Error;
use std::fmt;

use crate::{CatalogDigest, CatalogGeneration, CatalogGenerationError};

/// Failure to admit one exact successor to a current catalog snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogTransitionError {
    /// The current generation has no representable successor.
    GenerationExhausted {
        /// Exact checked-generation failure.
        source: CatalogGenerationError,
    },
    /// The candidate generation was not the exact successor.
    Generation {
        /// Required successor generation.
        expected: CatalogGeneration,
        /// Candidate generation.
        observed: CatalogGeneration,
    },
    /// The candidate did not name the current catalog digest.
    Predecessor {
        /// Required predecessor digest.
        expected: CatalogDigest,
        /// Candidate predecessor coordinate.
        observed: Option<CatalogDigest>,
    },
}

impl fmt::Display for CatalogTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationExhausted { source } => {
                write!(formatter, "catalog generation is exhausted: {source}")
            }
            Self::Generation { expected, observed } => write!(
                formatter,
                "catalog successor generation must be {}, observed {}",
                expected.get(),
                observed.get()
            ),
            Self::Predecessor { .. } => {
                formatter.write_str("catalog successor predecessor digest mismatch")
            }
        }
    }
}

impl Error for CatalogTransitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GenerationExhausted { source } => Some(source),
            _ => None,
        }
    }
}
