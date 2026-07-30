//! This module owns the recovery-inventory storage port.

use std::io;

use super::{RecoveryEntryName, RecoveryNamespace};

/// Read-only storage capabilities required to inventory recovery evidence.
///
/// Implementations must count without retaining names and stop after observing
/// `remaining + 1` entries. After count admission, `read_entry_names` must stop
/// and refuse if it observes more than `expected_count`; callers independently
/// verify the returned exact count.
pub trait RecoveryInventoryStorage {
    /// Counts one namespace up to the remaining global budget plus one.
    ///
    /// # Errors
    ///
    /// Returns the underlying storage refusal.
    fn count_entries(&mut self, namespace: RecoveryNamespace, remaining: u64) -> io::Result<u64>;

    /// Reads at most the previously observed number of validated raw names.
    ///
    /// # Errors
    ///
    /// Returns the underlying storage, validation, or drift refusal.
    fn read_entry_names(
        &mut self,
        namespace: RecoveryNamespace,
        expected_count: u64,
    ) -> io::Result<Vec<RecoveryEntryName>>;
}
