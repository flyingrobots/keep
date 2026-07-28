//! Configured complete-segment record admission bound.

use std::error::Error;
use std::fmt;

const PROTOCOL_MAXIMUM: u32 = 1_048_576;

/// Caller-selected maximum number of records admitted from one segment.
///
/// This resource policy does not alter canonical bytes or identity. A zero
/// limit admits only empty segments.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentRecordLimit(u32);

impl SegmentRecordLimit {
    /// The version-1 protocol maximum.
    pub const MAXIMUM: Self = Self(PROTOCOL_MAXIMUM);

    /// Constructs a record cap at or below the protocol maximum.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordLimitError`] when `value` exceeds the immutable
    /// version-1 format bound.
    pub const fn new(value: u32) -> Result<Self, SegmentRecordLimitError> {
        if value > PROTOCOL_MAXIMUM {
            return Err(SegmentRecordLimitError {
                maximum: PROTOCOL_MAXIMUM,
                observed: value,
            });
        }
        Ok(Self(value))
    }

    /// Returns the configured record count cap.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A requested segment-record cap exceeds the immutable protocol limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentRecordLimitError {
    maximum: u32,
    observed: u32,
}

impl SegmentRecordLimitError {
    /// Returns the immutable protocol maximum.
    #[must_use]
    pub const fn maximum(self) -> u32 {
        self.maximum
    }

    /// Returns the rejected requested cap.
    #[must_use]
    pub const fn observed(self) -> u32 {
        self.observed
    }
}

impl fmt::Display for SegmentRecordLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "segment record limit {} exceeds protocol maximum {}",
            self.observed, self.maximum
        )
    }
}

impl Error for SegmentRecordLimitError {}
