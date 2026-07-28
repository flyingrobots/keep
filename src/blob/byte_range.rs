//! Validated half-open logical byte range.

use std::error::Error;
use std::fmt;

use super::{ByteLength, ByteOffset};

/// A validated half-open logical byte range `[offset, end)`.
///
/// Construction proves that `offset + length` is representable. It does not
/// prove that the range is within any particular blob.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteRange {
    offset: ByteOffset,
    length: ByteLength,
    end: ByteOffset,
}

impl ByteRange {
    /// Constructs a range after checking its exclusive end.
    ///
    /// # Errors
    ///
    /// Returns [`ByteRangeError::EndOverflow`] when `offset + length` cannot
    /// be represented by `u64`.
    pub const fn new(offset: ByteOffset, length: ByteLength) -> Result<Self, ByteRangeError> {
        match offset.get().checked_add(length.get()) {
            Some(end) => Ok(Self {
                offset,
                length,
                end: ByteOffset::new(end),
            }),
            None => Err(ByteRangeError::EndOverflow { offset, length }),
        }
    }

    /// Returns the inclusive start coordinate.
    #[must_use]
    pub const fn offset(self) -> ByteOffset {
        self.offset
    }

    /// Returns the requested byte count.
    #[must_use]
    pub const fn length(self) -> ByteLength {
        self.length
    }

    /// Returns the exclusive end coordinate.
    #[must_use]
    pub const fn end(self) -> ByteOffset {
        self.end
    }

    /// Returns whether the range contains no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.length.is_empty()
    }
}

/// Failure while validating a logical byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteRangeError {
    /// The exclusive end cannot be represented.
    EndOverflow {
        /// Requested inclusive start.
        offset: ByteOffset,
        /// Requested byte count.
        length: ByteLength,
    },
}

impl fmt::Display for ByteRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOverflow { offset, length } => {
                write!(
                    formatter,
                    "byte range end overflow at offset {offset} with length {length}"
                )
            }
        }
    }
}

impl Error for ByteRangeError {}
