//! Explicit layout entry admission cap.

use std::error::Error;
use std::fmt;

const PROTOCOL_MAXIMUM: u32 = 1_048_576;

/// Caller-selected maximum number of entries admitted into memory.
///
/// This resource policy never changes canonical bytes or identity. A zero
/// limit admits only empty layouts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutEntryLimit(u32);

impl LayoutEntryLimit {
    /// The version-1 protocol maximum.
    pub const MAXIMUM: Self = Self(PROTOCOL_MAXIMUM);

    /// Constructs an entry cap at or below the protocol maximum.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutEntryLimitError`] when `value` exceeds the immutable
    /// version-1 wire bound.
    pub const fn new(value: u32) -> Result<Self, LayoutEntryLimitError> {
        if value > PROTOCOL_MAXIMUM {
            return Err(LayoutEntryLimitError {
                maximum: PROTOCOL_MAXIMUM,
                observed: value,
            });
        }
        Ok(Self(value))
    }

    /// Returns the configured entry count cap.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A requested entry cap exceeds the immutable protocol limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutEntryLimitError {
    maximum: u32,
    observed: u32,
}

impl LayoutEntryLimitError {
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

impl fmt::Display for LayoutEntryLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "layout entry limit {} exceeds protocol maximum {}",
            self.observed, self.maximum
        )
    }
}

impl Error for LayoutEntryLimitError {}
