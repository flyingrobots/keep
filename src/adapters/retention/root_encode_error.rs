//! This boundary module owns typed retention root encoding failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

/// Failure to materialize one canonical retention root record.
#[derive(Debug)]
pub enum RetentionRootEncodeError {
    /// Checked record-length arithmetic overflowed.
    LengthOverflow,
    /// Exact record allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
    /// Construction produced a length different from its admitted plan.
    ConstructionLength {
        /// Planned exact length.
        expected: usize,
        /// Materialized length.
        observed: usize,
    },
}

impl fmt::Display for RetentionRootEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => formatter.write_str("retention root record length overflow"),
            Self::Allocation { .. } => {
                formatter.write_str("retention root record allocation failed")
            }
            Self::ConstructionLength { expected, observed } => write!(
                formatter,
                "retention root construction produced {observed} bytes; expected {expected}"
            ),
        }
    }
}

impl Error for RetentionRootEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source } => Some(source),
            Self::LengthOverflow | Self::ConstructionLength { .. } => None,
        }
    }
}
