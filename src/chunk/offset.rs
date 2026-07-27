//! Absolute chunk offset within one input stream.

use std::fmt;

/// An absolute zero-based byte coordinate in one detector input stream.
///
/// This coordinate is not content identity and is not a physical storage
/// location.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkOffset(u64);

impl ChunkOffset {
    /// The beginning of a stream.
    pub const ZERO: Self = Self(0);

    /// Returns the absolute byte coordinate.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(super) const fn checked_increment(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub(crate) const fn from_validated(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ChunkOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
