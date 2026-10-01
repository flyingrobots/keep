//! This boundary module owns the count-and-byte limits staging admits.

use std::fmt;

use crate::LayoutEntryLimit;

/// The most bytes one staging may accept from its source.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StagedByteLimit(u64);

impl StagedByteLimit {
    /// No bound beyond the backend's own capacity.
    pub const MAXIMUM: Self = Self(u64::MAX);

    /// A bound of exactly `bytes`.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// The bound.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for StagedByteLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} bytes", self.0)
    }
}

/// What one staging admits: at most `entries` layout entries and at most
/// `bytes` source bytes. Both refuse before the excess is materialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StagingLimits {
    entries: LayoutEntryLimit,
    bytes: StagedByteLimit,
}

impl StagingLimits {
    /// Limits of exactly `entries` and `bytes`.
    #[must_use]
    pub const fn new(entries: LayoutEntryLimit, bytes: StagedByteLimit) -> Self {
        Self { entries, bytes }
    }

    /// The entry limit alone, with no byte bound beyond the backend's.
    #[must_use]
    pub const fn entries(entries: LayoutEntryLimit) -> Self {
        Self::new(entries, StagedByteLimit::MAXIMUM)
    }

    /// The layout entry limit.
    #[must_use]
    pub const fn entry_limit(self) -> LayoutEntryLimit {
        self.entries
    }

    /// The source byte limit.
    #[must_use]
    pub const fn byte_limit(self) -> StagedByteLimit {
        self.bytes
    }
}
