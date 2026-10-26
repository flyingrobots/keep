//! This module owns garbage-collection generation admission failures.

use std::error::Error;
use std::fmt;

/// Failure to admit or advance one garbage-collection generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcGenerationError {
    /// Generation zero is never a retirement.
    Zero,
    /// The generation space is exhausted.
    Exhausted {
        /// Current maximum generation.
        current: u64,
    },
}

impl fmt::Display for GcGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("garbage-collection generation must be positive"),
            Self::Exhausted { current } => write!(
                formatter,
                "garbage-collection generation {current} has no successor"
            ),
        }
    }
}

impl Error for GcGenerationError {}
