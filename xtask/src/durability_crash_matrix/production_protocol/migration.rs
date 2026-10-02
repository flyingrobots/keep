//! This module owns execution of the production store-migration protocol.

use std::path::Path;

use keep::{FilesystemStoreMigrationAuthority, FilesystemWriterLock, execute_store_migration};

use super::control::CrashControl;
use super::initialization;
use super::migration_storage::CrashMigrationStorage;
use super::publication;
use super::{DurabilityCrashMatrixError, verification};

/// Publishes the Golden File Worldline version-1 store, then migrates it
/// through the production 21-phase protocol with the selected boundary gated
/// for process death.
///
/// No version-1 gate fires for a migration case, so the publication
/// precondition runs to completion and releases its writer lock before the
/// migration authority reacquires it.
pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    publication::run(store_root, control)?;
    let lock = FilesystemWriterLock::try_acquire(store_root)
        .map_err(|source| verification("reacquire writer lock for migration", source))?;
    let authority = FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks(
        lock,
        initialization::segment_policy(),
    )
    .map_err(|source| verification("open production migration authority", source))?;
    let intent = authority
        .observe_intent()
        .map_err(|source| verification("observe production migration intent", source))?;
    let mut storage = CrashMigrationStorage::new(authority, control);
    execute_store_migration(&mut storage, &intent)
        .map(|_receipt| ())
        .map_err(|source| verification("execute production store migration", source))
}
