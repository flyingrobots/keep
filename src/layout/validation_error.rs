//! Semantic layout validation failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use crate::BlobLength;

/// Failure to construct an admitted semantic flat layout.
#[derive(Debug)]
pub enum LayoutValidationError {
    /// The entry count exceeds the caller's explicit admission cap.
    EntryLimitExceeded {
        /// Configured maximum entry count.
        maximum: u32,
        /// Observed materialized entry count.
        observed: usize,
    },
    /// The configured entry cap does not fit the host index width.
    EntryLimitHostWidth {
        /// Configured cap that cannot be represented by `usize`.
        observed: u32,
    },
    /// A host allocation for admitted entries failed.
    Allocation {
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// An empty target declared one or more entries.
    EmptyBlobHasEntries {
        /// Observed entry count.
        observed: usize,
    },
    /// A nonempty target declared no entries.
    NonemptyBlobHasNoEntries,
    /// The first entry did not begin at logical offset zero.
    FirstOffsetNotZero {
        /// Observed first logical offset.
        observed: u64,
    },
    /// An entry begins after the prior exclusive end.
    Gap {
        /// Zero-based entry index.
        index: u32,
        /// Required next logical offset.
        expected: u64,
        /// Observed logical offset.
        observed: u64,
    },
    /// An entry begins before the prior exclusive end.
    Overlap {
        /// Zero-based entry index.
        index: u32,
        /// Required next logical offset.
        expected: u64,
        /// Observed logical offset.
        observed: u64,
    },
    /// A chunk length violates the admitted profile's position-specific bound.
    ProfileLengthOutOfBounds {
        /// Zero-based entry index.
        index: u32,
        /// Minimum admitted length at this position.
        minimum: u32,
        /// Maximum admitted length.
        maximum: u32,
        /// Observed chunk length.
        observed: u32,
    },
    /// An entry's exclusive end cannot be represented by `u64`.
    OffsetOverflow {
        /// Zero-based entry index.
        index: u32,
        /// Entry start offset.
        offset: u64,
        /// Entry byte length.
        length: u32,
    },
    /// The final exclusive end differs from the target logical length.
    AggregateLengthMismatch {
        /// Logical length committed by the target blob identity.
        expected: BlobLength,
        /// Final exclusive end computed from entries.
        observed: u64,
    },
    /// An entry index cannot be represented by the version-1 `u32` count.
    EntryIndexOutOfRange {
        /// Host index that cannot be represented.
        observed: usize,
    },
}

impl fmt::Display for LayoutValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryLimitExceeded { maximum, observed } => write!(
                formatter,
                "layout has {observed} entries above configured limit {maximum}"
            ),
            Self::EntryLimitHostWidth { observed } => {
                write!(
                    formatter,
                    "layout entry limit {observed} exceeds host index width"
                )
            }
            Self::Allocation { .. } => formatter.write_str("layout entry allocation failed"),
            Self::EmptyBlobHasEntries { observed } => {
                write!(formatter, "empty blob layout has {observed} entries")
            }
            Self::NonemptyBlobHasNoEntries => {
                formatter.write_str("nonempty blob layout has no entries")
            }
            Self::FirstOffsetNotZero { observed } => {
                write!(formatter, "first layout offset is {observed}, not zero")
            }
            Self::Gap {
                index,
                expected,
                observed,
            } => write!(
                formatter,
                "layout gap at entry {index}: expected offset {expected}, observed {observed}"
            ),
            Self::Overlap {
                index,
                expected,
                observed,
            } => write!(
                formatter,
                "layout overlap at entry {index}: expected offset {expected}, observed {observed}"
            ),
            Self::ProfileLengthOutOfBounds {
                index,
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "layout entry {index} length {observed} is outside {minimum}..={maximum}"
            ),
            Self::OffsetOverflow {
                index,
                offset,
                length,
            } => write!(
                formatter,
                "layout entry {index} end overflows from offset {offset} and length {length}"
            ),
            Self::AggregateLengthMismatch { expected, observed } => write!(
                formatter,
                "layout aggregate length mismatch: expected {expected}, observed {observed}"
            ),
            Self::EntryIndexOutOfRange { observed } => {
                write!(formatter, "layout entry index {observed} exceeds u32")
            }
        }
    }
}

impl Error for LayoutValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source } => Some(source),
            _ => None,
        }
    }
}
