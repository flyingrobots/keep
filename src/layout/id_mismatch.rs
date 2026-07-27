//! Layout identity comparison failures.

use std::error::Error;
use std::fmt;

use super::LayoutRecordLength;

/// Failure to match an observed `LayoutId` with an expected coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutIdMismatch {
    /// The identities commit to different canonical record lengths.
    PlanLength {
        /// Expected record length.
        expected: LayoutRecordLength,
        /// Observed record length.
        observed: LayoutRecordLength,
    },
    /// Record lengths match but identity digests differ.
    Digest {
        /// Expected BLAKE3-256 identity digest.
        expected: [u8; 32],
        /// Observed BLAKE3-256 identity digest.
        observed: [u8; 32],
    },
}

impl fmt::Display for LayoutIdMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanLength { expected, observed } => write!(
                formatter,
                "layout identity plan length mismatch: expected {expected}, observed {observed}"
            ),
            Self::Digest { .. } => formatter.write_str("layout identity digest mismatch"),
        }
    }
}

impl Error for LayoutIdMismatch {}
