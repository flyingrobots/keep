//! This module owns canonical retention manifest byte lengths.

use super::RetentionManifestLengthError;

const HEADER_LENGTH: u64 = 160;
const ENTRY_LENGTH: u64 = 72;
const TRAILER_LENGTH: u64 = 64;
const MINIMUM_VALUE: u64 = HEADER_LENGTH + TRAILER_LENGTH;
const MAXIMUM_VALUE: u64 = 295_136;

/// Exact canonical byte length of one complete version-2 retention manifest.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionManifestLength(u64);

impl RetentionManifestLength {
    /// Smallest complete version-2 retention manifest length.
    pub const MINIMUM: Self = Self(MINIMUM_VALUE);

    /// Largest complete version-2 retention manifest length.
    pub const MAXIMUM: Self = Self(MAXIMUM_VALUE);

    /// Admits one complete canonical retention manifest length.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionManifestLengthError`] when `value` exceeds the
    /// format bound or cannot contain a whole number of fixed-width entries.
    pub const fn new(value: u64) -> Result<Self, RetentionManifestLengthError> {
        if value < MINIMUM_VALUE || value > MAXIMUM_VALUE {
            return Err(RetentionManifestLengthError::OutOfBounds {
                minimum: MINIMUM_VALUE,
                maximum: MAXIMUM_VALUE,
                observed: value,
            });
        }
        let Some(entry_bytes) = value.checked_sub(MINIMUM_VALUE) else {
            return Err(RetentionManifestLengthError::OutOfBounds {
                minimum: MINIMUM_VALUE,
                maximum: MAXIMUM_VALUE,
                observed: value,
            });
        };
        if !entry_bytes.is_multiple_of(ENTRY_LENGTH) {
            return Err(RetentionManifestLengthError::NotCongruent { observed: value });
        }
        Ok(Self(value))
    }

    /// Returns the exact admitted byte length.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
