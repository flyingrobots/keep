//! Canonical layout encoding failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

/// Failure to encode an admitted layout into its canonical record.
#[derive(Debug)]
pub enum LayoutEncodeError {
    /// The admitted entry count cannot be represented by the wire grammar.
    EntryCountOutOfRange {
        /// Host entry count that could not be converted to `u32`.
        observed: usize,
        /// Original integer conversion failure.
        source: TryFromIntError,
    },
    /// Checked record-length arithmetic failed or produced an invalid length.
    RecordLengthOutOfRange {
        /// Entry count used to calculate the record length.
        entry_count: u32,
    },
    /// The canonical record length cannot be represented by the host.
    HostLengthOutOfRange {
        /// Canonical wire length that could not be converted to `usize`.
        observed: u64,
        /// Original integer conversion failure.
        source: TryFromIntError,
    },
    /// Allocating the exact canonical record buffer failed.
    Allocation {
        /// Exact requested byte count.
        requested: usize,
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// The encoder emitted a length different from its checked plan.
    InvariantLength {
        /// Planned canonical record length.
        expected: u64,
        /// Emitted host byte count.
        observed: usize,
    },
}

impl fmt::Display for LayoutEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCountOutOfRange { observed, .. } => {
                write!(formatter, "layout entry count {observed} exceeds u32")
            }
            Self::RecordLengthOutOfRange { entry_count } => write!(
                formatter,
                "layout record length is invalid for {entry_count} entries"
            ),
            Self::HostLengthOutOfRange { observed, .. } => {
                write!(
                    formatter,
                    "layout record length {observed} exceeds host width"
                )
            }
            Self::Allocation { requested, .. } => {
                write!(
                    formatter,
                    "allocation of {requested} layout record bytes failed"
                )
            }
            Self::InvariantLength { expected, observed } => write!(
                formatter,
                "layout encoder emitted {observed} bytes, expected {expected}"
            ),
        }
    }
}

impl Error for LayoutEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EntryCountOutOfRange { source, .. }
            | Self::HostLengthOutOfRange { source, .. } => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::RecordLengthOutOfRange { .. } | Self::InvariantLength { .. } => None,
        }
    }
}
