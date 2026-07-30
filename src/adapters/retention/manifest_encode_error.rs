//! This boundary module owns retention manifest encoding failures.

use std::{collections::TryReserveError, error::Error, fmt};

/// Failure to encode one canonical retention manifest record.
#[derive(Debug)]
pub enum RetentionManifestEncodeError {
    /// Checked record-length arithmetic overflowed.
    LengthOverflow,
    /// Canonical byte allocation was refused.
    Allocation {
        /// Preserved allocation failure.
        source: TryReserveError,
    },
    /// Internal construction produced a noncanonical length.
    ConstructionLength {
        /// Required length.
        expected: usize,
        /// Constructed length.
        observed: usize,
    },
}

impl fmt::Display for RetentionManifestEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => formatter.write_str("retention manifest length overflow"),
            Self::Allocation { .. } => formatter.write_str("retention manifest allocation failed"),
            Self::ConstructionLength { expected, observed } => write!(
                formatter,
                "retention manifest construction produced {observed} bytes; expected {expected}"
            ),
        }
    }
}

impl Error for RetentionManifestEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source } => Some(source),
            Self::LengthOverflow | Self::ConstructionLength { .. } => None,
        }
    }
}
