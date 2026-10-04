//! This module owns typed retention liveness-generation failures.

use std::error::Error;
use std::fmt;

/// Failure to admit or advance a retention liveness generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LivenessGenerationError {
    /// Generation zero is outside the version-2 protocol.
    Zero,
    /// The current generation has no representable successor.
    Exhausted {
        /// Exact generation that could not advance.
        current: u64,
    },
}

impl fmt::Display for LivenessGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("retention liveness generation must be positive"),
            Self::Exhausted { current } => write!(
                formatter,
                "retention liveness generation {current} has no successor"
            ),
        }
    }
}

impl Error for LivenessGenerationError {}
