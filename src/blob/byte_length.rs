//! Exact logical byte-range length.

use std::fmt;

/// An exact finite logical byte count for one requested range.
///
/// Zero is lawful. This value is not a complete blob length, an allocation
/// size, or a physical extent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteLength(u64);

impl ByteLength {
    /// The length of an empty range.
    pub const ZERO: Self = Self(0);

    /// Constructs an exact requested byte count.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact requested byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns whether this byte count is zero.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for ByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
