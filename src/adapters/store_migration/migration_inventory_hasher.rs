//! This boundary module owns streamed canonical migration inventory identity.

use super::{
    ImmutablePoolInventoryDigest, StoreMigrationInventoryEntry, StoreMigrationInventoryEntryCount,
    StoreMigrationInventoryError,
};

const DOMAIN: &[u8] = b"keep.store-v1-pool-inventory/v2\0";

/// In-progress bounded digest over one declared canonical pool inventory.
///
/// Entries must be supplied in complete canonical byte order. The hasher
/// retains only the preceding entry and never materializes the complete
/// encoded inventory.
#[must_use]
pub struct StoreMigrationInventoryHasher {
    expected: StoreMigrationInventoryEntryCount,
    observed: u32,
    previous: Option<StoreMigrationInventoryEntry>,
    hasher: blake3::Hasher,
}

impl StoreMigrationInventoryHasher {
    /// Begins one inventory whose exact count is known before entry streaming.
    pub fn new(expected: StoreMigrationInventoryEntryCount) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(DOMAIN);
        hasher.update(&expected.get().to_be_bytes());
        Self {
            expected,
            observed: 0,
            previous: None,
            hasher,
        }
    }

    /// Admits and hashes the next exact canonical entry.
    ///
    /// # Errors
    ///
    /// Returns [`StoreMigrationInventoryError`] for count excess, a duplicate,
    /// noncanonical order, or checked count overflow.
    pub fn push(
        &mut self,
        entry: StoreMigrationInventoryEntry,
    ) -> Result<(), StoreMigrationInventoryError> {
        let observed = self
            .observed
            .checked_add(1)
            .ok_or(StoreMigrationInventoryError::EntryCountOverflow)?;
        if observed > self.expected.get() {
            return Err(StoreMigrationInventoryError::EntryCountExceeded {
                expected: self.expected,
                observed,
            });
        }
        if let Some(previous) = self.previous {
            if entry == previous {
                return Err(StoreMigrationInventoryError::Duplicate { entry });
            }
            if entry < previous {
                return Err(StoreMigrationInventoryError::OutOfOrder {
                    previous,
                    observed: entry,
                });
            }
        }
        self.hasher.update(entry.encoded());
        self.previous = Some(entry);
        self.observed = observed;
        Ok(())
    }

    /// Finalizes only after the declared number of entries was admitted.
    ///
    /// # Errors
    ///
    /// Returns [`StoreMigrationInventoryError::Incomplete`] when fewer entries
    /// were supplied than declared.
    pub fn finish(self) -> Result<ImmutablePoolInventoryDigest, StoreMigrationInventoryError> {
        if self.observed != self.expected.get() {
            return Err(StoreMigrationInventoryError::Incomplete {
                expected: self.expected,
                observed: self.observed,
            });
        }
        Ok(ImmutablePoolInventoryDigest::from_admitted(
            *self.hasher.finalize().as_bytes(),
        ))
    }
}
