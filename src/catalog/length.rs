//! Checked canonical catalog byte length.

use super::CatalogLengthError;

const HEADER_LENGTH: u64 = 128;
const ENTRY_LENGTH: u64 = 160;
const TRAILER_LENGTH: u64 = 64;
const MINIMUM_VALUE: u64 = HEADER_LENGTH + TRAILER_LENGTH;
const MAXIMUM_VALUE: u64 = 167_772_352;

/// Exact canonical byte length of one complete version-1 catalog.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogLength(u64);

impl CatalogLength {
    /// Smallest complete version-1 catalog length.
    pub const MINIMUM: Self = Self(MINIMUM_VALUE);

    /// Largest complete version-1 catalog length.
    pub const MAXIMUM: Self = Self(MAXIMUM_VALUE);

    /// Admits a complete version-1 catalog length.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogLengthError`] when `value` exceeds the format bound or
    /// cannot contain a whole number of fixed-width entries.
    pub const fn new(value: u64) -> Result<Self, CatalogLengthError> {
        if value < MINIMUM_VALUE || value > MAXIMUM_VALUE {
            return Err(CatalogLengthError::OutOfBounds {
                minimum: MINIMUM_VALUE,
                maximum: MAXIMUM_VALUE,
                observed: value,
            });
        }
        let Some(entry_bytes) = value.checked_sub(MINIMUM_VALUE) else {
            return Err(CatalogLengthError::OutOfBounds {
                minimum: MINIMUM_VALUE,
                maximum: MAXIMUM_VALUE,
                observed: value,
            });
        };
        if !entry_bytes.is_multiple_of(ENTRY_LENGTH) {
            return Err(CatalogLengthError::NotCongruent { observed: value });
        }
        Ok(Self(value))
    }

    /// Returns the exact admitted byte length.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
