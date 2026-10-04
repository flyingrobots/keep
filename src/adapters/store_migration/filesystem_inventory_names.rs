//! This module owns deterministic bounded migration pool-name scans.

use cap_std::fs::Dir;

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};
use crate::adapters::{RecoveryEntryName, filesystem_recovery_inventory_scan};

pub(super) fn read(
    directory: &Dir,
    pool: MigrationInventoryPool,
    remaining: u32,
) -> Result<Vec<RecoveryEntryName>, FilesystemMigrationInventoryError> {
    let expected =
        filesystem_recovery_inventory_scan::count_entries(directory, u64::from(remaining))
            .map_err(|source| FilesystemMigrationInventoryError::Io {
                namespace: MigrationInventoryNamespace::from(pool),
                operation: FilesystemMigrationInventoryOperation::CountEntries,
                source,
            })?;
    if expected > u64::from(remaining) {
        return Err(FilesystemMigrationInventoryError::EntryLimitExceeded {
            pool,
            maximum: remaining,
            observed_at_least: expected,
        });
    }
    let mut names = filesystem_recovery_inventory_scan::read_entry_names(directory, expected)
        .map_err(|source| FilesystemMigrationInventoryError::Io {
            namespace: MigrationInventoryNamespace::from(pool),
            operation: FilesystemMigrationInventoryOperation::ReadEntryNames,
            source,
        })?;
    let observed = u64::try_from(names.len())
        .map_err(|_source| FilesystemMigrationInventoryError::EntryCountHostWidth { pool })?;
    if observed != expected {
        return Err(FilesystemMigrationInventoryError::EntryCountChanged {
            pool,
            expected,
            observed,
        });
    }
    names.sort_unstable();
    Ok(names)
}

pub(super) fn verify(
    directory: &Dir,
    pool: MigrationInventoryPool,
    remaining: u32,
    expected: &[RecoveryEntryName],
) -> Result<(), FilesystemMigrationInventoryError> {
    let observed = read(directory, pool, remaining)?;
    if observed == expected {
        Ok(())
    } else {
        Err(FilesystemMigrationInventoryError::EntriesChanged { pool })
    }
}
