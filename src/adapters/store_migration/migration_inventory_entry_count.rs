//! This module owns the bounded migration inventory entry count.

use super::StoreMigrationInventoryEntryCountError;

/// Exact number of canonical entries expected in one migration inventory.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreMigrationInventoryEntryCount(u32);

impl StoreMigrationInventoryEntryCount {
    /// Largest number of immutable-pool entries admitted by version 2.
    pub const MAXIMUM: u32 = 2_097_152;

    /// Admits one exact entry count, including an empty inventory.
    ///
    /// # Errors
    ///
    /// Returns [`StoreMigrationInventoryEntryCountError`] above
    /// [`Self::MAXIMUM`].
    pub const fn new(value: u32) -> Result<Self, StoreMigrationInventoryEntryCountError> {
        if value <= Self::MAXIMUM {
            Ok(Self(value))
        } else {
            Err(StoreMigrationInventoryEntryCountError::AboveMaximum {
                observed: value,
                maximum: Self::MAXIMUM,
            })
        }
    }

    /// Returns the exact admitted count.
    pub const fn get(self) -> u32 {
        self.0
    }
}
