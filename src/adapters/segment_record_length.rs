//! Admitted complete segment-record length.

use std::fmt;

/// Exact byte count of one admitted complete segment record.
///
/// The value includes the fixed header, exact payload, and fixed checksum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentRecordLength(u64);

impl SegmentRecordLength {
    pub(super) const fn from_validated(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact complete record byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SegmentRecordLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
