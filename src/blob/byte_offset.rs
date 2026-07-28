//! Absolute logical byte offset.

use std::fmt;

/// An absolute zero-based coordinate within one logical blob.
///
/// This coordinate is not content identity, a chunk offset, or a physical
/// storage location. Every `u64` value is a lawful coordinate; a
/// [`ByteRange`](super::ByteRange) validates arithmetic and a layout validates
/// whether the coordinate is in bounds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteOffset(u64);

impl ByteOffset {
    /// The beginning of a logical blob.
    pub const ZERO: Self = Self(0);

    /// Constructs an absolute logical coordinate.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the absolute zero-based coordinate.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
