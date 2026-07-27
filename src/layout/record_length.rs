//! Canonical flat-layout record length.

use std::fmt;

const HEADER_AND_CHECKSUM_LENGTH: u64 = 176;
const ENTRY_LENGTH: u64 = 44;
const MAXIMUM_RECORD_LENGTH: u64 = 46_137_520;

/// Exact byte length of one canonical flat-layout record.
///
/// Every value is within the version-1 wire bound and congruent with the
/// fixed header, entry, and checksum widths.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutRecordLength(u64);

impl LayoutRecordLength {
    /// Smallest canonical record length: an empty layout.
    pub const MINIMUM: u64 = HEADER_AND_CHECKSUM_LENGTH;
    /// Largest canonical record length under the version-1 entry limit.
    pub const MAXIMUM: u64 = MAXIMUM_RECORD_LENGTH;

    pub(crate) const fn from_wire(value: u64) -> Option<Self> {
        if value < Self::MINIMUM || value > Self::MAXIMUM {
            return None;
        }
        let Some(entry_bytes) = value.checked_sub(HEADER_AND_CHECKSUM_LENGTH) else {
            return None;
        };
        if !entry_bytes.is_multiple_of(ENTRY_LENGTH) {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the exact encoded byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for LayoutRecordLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
