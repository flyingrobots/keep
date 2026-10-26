//! These laws own runtime recovery of typed filesystem-operation refusals.
//! Size: small to medium (owned filesystem). Oracle: callers recover the exact
//! refusal and evidence by downcasting the `io::Error` payload.
//! Delete only when these ports stop returning `io::Error`.

use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::FilesystemOperationRefusal as Refusal;
use crate::adapters::filesystem_initialization_storage::FilesystemInitializationStorage;
use crate::adapters::filesystem_recovery_inventory_scan::{count_entries, read_entry_names};
use crate::adapters::filesystem_recovery_namespace::PinnedRecoveryDirectory;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{
    FilesystemEntryIdentity, FilesystemEntryKind, RecoveryInventoryError, RecoveryInventoryLimit,
    RecoveryNamespace, StoreInitializationStorage,
};

fn operation_refusal<T>(result: io::Result<T>) -> Option<(io::ErrorKind, Refusal)> {
    let error = result.err()?;
    let refusal = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<Refusal>())
        .copied()?;
    Some((error.kind(), refusal))
}

fn inventory_refusal<T>(
    result: Result<T, RecoveryInventoryError>,
) -> Option<(io::ErrorKind, Refusal)> {
    match result.err()? {
        RecoveryInventoryError::Io { source, .. } => operation_refusal::<()>(Err(source)),
        _ => None,
    }
}

fn identity(path: &Path) -> io::Result<FilesystemEntryIdentity> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(FilesystemEntryIdentity::new(metadata.dev(), metadata.ino()))
}

#[test]
fn initialization_mutations_without_writer_authority_refuse_typed() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("operation-refusal-initialization")?;
    let absent = Some((io::ErrorKind::Other, Refusal::WriterAuthorityAbsent));

    let mut storage = FilesystemInitializationStorage::admit_unchecked_for_tests(sandbox.path())?;
    assert_eq!(operation_refusal(storage.synchronize_root()), absent);
    assert_eq!(operation_refusal(storage.admit_staging_directory()), absent);
    assert_eq!(operation_refusal(storage.into_lock()), absent);

    let mut locked = FilesystemInitializationStorage::admit_unchecked_for_tests(sandbox.path())?;
    locked.open_and_lock_writer_file()?;
    assert_eq!(
        operation_refusal(locked.open_and_lock_writer_file()),
        Some((io::ErrorKind::Other, Refusal::WriterAuthorityAlreadyAcquired))
    );
    drop(locked);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn recovery_counts_above_the_protocol_maximum_refuse_with_both_bounds() -> Result<(), Box<dyn Error>>
{
    let sandbox = TestDirectory::create("operation-refusal-inventory-limit")?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let maximum = RecoveryInventoryLimit::PROTOCOL_MAXIMUM;
    let observed = maximum.checked_add(1).ok_or("protocol maximum has no successor")?;
    let refused = Some((
        io::ErrorKind::InvalidInput,
        Refusal::InventoryLimit { maximum, observed },
    ));

    assert_eq!(operation_refusal(count_entries(&directory, observed)), refused);
    assert_eq!(operation_refusal(read_entry_names(&directory, observed)), refused);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_replaced_pinned_recovery_namespace_refuses_with_both_identities() -> Result<(), Box<dyn Error>>
{
    let sandbox = TestDirectory::create("operation-refusal-recovery-identity")?;
    let staging = sandbox.path().join("staging");
    fs::create_dir(&staging)?;
    let root = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let pinned = PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Staging, "staging")?;
    let expected = identity(&staging)?;
    fs::rename(&staging, sandbox.path().join("displaced"))?;
    fs::create_dir(&staging)?;
    let observed = identity(&staging)?;

    assert_eq!(
        inventory_refusal(pinned.verify(&root)),
        Some((
            io::ErrorKind::InvalidData,
            Refusal::IdentityChanged { expected, observed }
        ))
    );

    fs::remove_dir(&staging)?;
    fs::write(&staging, [])?;
    assert_eq!(
        inventory_refusal(pinned.verify(&root)),
        Some((
            io::ErrorKind::InvalidData,
            Refusal::DirectoryRequired {
                observed: FilesystemEntryKind::File
            }
        ))
    );
    drop(pinned);
    sandbox.remove()?;
    Ok(())
}
