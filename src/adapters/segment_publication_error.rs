//! This module owns closed-stage to admitted-segment binding failures.

use std::error::Error;
use std::fmt;

use super::SegmentDigest;

/// A closed stage receipt disagreed with the admitted segment selected for publication.
#[derive(Debug)]
pub enum SegmentPublicationError {
    /// The admitted segment byte length cannot be represented by the protocol.
    HostLength {
        /// Host byte length that could not be represented.
        observed: usize,
    },
    /// The closed stage and admitted segment have different record counts.
    RecordCount {
        /// Record count proven before close.
        expected: u32,
        /// Record count verified from admitted bytes.
        observed: u32,
    },
    /// The closed stage and admitted segment have different byte lengths.
    SegmentLength {
        /// Byte length proven before close.
        expected: u64,
        /// Byte length verified from admitted bytes.
        observed: u64,
    },
    /// The closed stage and admitted segment have different physical digests.
    Digest {
        /// Digest proven before close.
        expected: SegmentDigest,
        /// Digest verified from admitted bytes.
        observed: SegmentDigest,
    },
}

impl fmt::Display for SegmentPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostLength { .. } => {
                formatter.write_str("admitted segment length is not representable")
            }
            Self::RecordCount { .. } => {
                formatter.write_str("closed and admitted segment record counts differ")
            }
            Self::SegmentLength { .. } => {
                formatter.write_str("closed and admitted segment lengths differ")
            }
            Self::Digest { .. } => {
                formatter.write_str("closed and admitted segment digests differ")
            }
        }
    }
}

impl Error for SegmentPublicationError {}
