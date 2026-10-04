//! Migration pool-name revalidation laws.

use std::error::Error;
use std::fs;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, MigrationInventoryPool,
};
use super::filesystem_inventory_names;
use crate::adapters::filesystem_test_sandbox::TestDirectory;

#[test]
fn membership_change_after_scan_refuses_revalidation() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-name-revalidation")?;
    fs::write(sandbox.path().join("first"), b"one")?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let expected =
        filesystem_inventory_names::read(&directory, MigrationInventoryPool::Segments, 2)?;
    fs::write(sandbox.path().join("second"), b"two")?;

    let error = filesystem_inventory_names::verify(
        &directory,
        MigrationInventoryPool::Segments,
        2,
        &expected,
    )
    .err()
    .ok_or("changed membership unexpectedly passed")?;
    assert!(matches!(
        error,
        FilesystemMigrationInventoryError::EntriesChanged {
            pool: MigrationInventoryPool::Segments
        }
    ));
    drop(directory);
    sandbox.remove()?;
    Ok(())
}
