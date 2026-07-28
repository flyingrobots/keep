//! Typed failures while mapping a logical range onto an admitted layout.

use std::error::Error;
use std::fmt;

use crate::{BlobLength, ByteRange, ChunkLength, ChunkOffset};

/// Failure while mapping a logical byte range onto an admitted layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangePlanError {
    /// The requested range extends beyond its target blob.
    OutOfBounds {
        /// Requested logical range.
        requested: ByteRange,
        /// Exact target blob length.
        target_length: BlobLength,
    },
    /// A layout entry's exclusive end cannot be represented.
    EntryEndOverflow {
        /// Zero-based layout-entry index.
        index: usize,
        /// Entry start coordinate.
        offset: ChunkOffset,
        /// Entry byte count.
        length: ChunkLength,
    },
    /// Advancing an overlapping entry index overflowed.
    EntryIndexOverflow {
        /// Last overlapping entry index.
        index: usize,
    },
    /// A nonempty in-bounds range found no overlapping entry.
    NoOverlap {
        /// Requested logical range.
        requested: ByteRange,
        /// Exact target blob length.
        target_length: BlobLength,
    },
    /// The calculated entry interval was inverted.
    EntryIntervalInverted {
        /// First overlapping entry.
        first: usize,
        /// Exclusive entry interval end.
        end: usize,
    },
}

impl fmt::Display for RangePlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds {
                requested,
                target_length,
            } => write!(
                formatter,
                "byte range [{}, {}) exceeds blob length {target_length}",
                requested.offset(),
                requested.end()
            ),
            Self::EntryEndOverflow {
                index,
                offset,
                length,
            } => write!(
                formatter,
                "layout entry {index} end overflow at offset {offset} with length {length}"
            ),
            Self::EntryIndexOverflow { index } => {
                write!(formatter, "layout entry index overflow after {index}")
            }
            Self::NoOverlap {
                requested,
                target_length,
            } => write!(
                formatter,
                "nonempty byte range [{}, {}) found no entry in blob length {target_length}",
                requested.offset(),
                requested.end()
            ),
            Self::EntryIntervalInverted { first, end } => write!(
                formatter,
                "range plan entry interval is inverted: first {first}, end {end}"
            ),
        }
    }
}

impl Error for RangePlanError {}
