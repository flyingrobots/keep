//! Typed catalog-generation admission and transition failures.

use std::error::Error;
use std::fmt;

/// Failure to admit or advance a catalog generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogGenerationError {
    /// Generation zero is outside the version-1 protocol.
    Zero,
    /// The current generation has no representable successor.
    Exhausted {
        /// The exact generation that could not advance.
        current: u64,
    },
}

impl fmt::Display for CatalogGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("catalog generation must be positive"),
            Self::Exhausted { current } => {
                write!(formatter, "catalog generation {current} has no successor")
            }
        }
    }
}

impl Error for CatalogGenerationError {}
