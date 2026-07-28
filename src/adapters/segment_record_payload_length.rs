//! Admitted segment-record payload length.

use std::fmt;

/// Exact payload byte count declared by an admitted segment record.
///
/// The value is positive and no greater than the version-1 record-payload
/// bound. Kind-specific admission may enforce a lower maximum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentRecordPayloadLength(u64);

impl SegmentRecordPayloadLength {
    pub(super) const fn from_validated(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact payload byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SegmentRecordPayloadLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
