//! Caller-selected aggregate restart segment-byte bound.

use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;

/// Positive maximum segment bytes retained by one restart snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogRestartByteLimit(NonZeroU64);

impl CatalogRestartByteLimit {
    /// Admits one positive aggregate byte bound.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogRestartByteLimitError`] when `value` is zero.
    pub const fn new(value: u64) -> Result<Self, CatalogRestartByteLimitError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(CatalogRestartByteLimitError),
        }
    }

    /// Returns the exact caller-selected bound.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Refusal of a zero aggregate restart byte bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogRestartByteLimitError;

impl fmt::Display for CatalogRestartByteLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("catalog restart byte limit must be positive")
    }
}

impl Error for CatalogRestartByteLimitError {}
