//! These laws own runtime recovery of typed forward-migration refusals.
//! Size: medium (owned filesystem). Oracle: a caller recovers the exact refusal
//! and its evidence by downcasting the `io::Error` payload.
//! Delete only when migration storage stops returning `io::Error`.

use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_migration_fixed_artifact::{
    FilesystemMigrationFixedArtifact as Artifact, FilesystemMigrationFixedStage,
};
use super::filesystem_migration_namespace_directory::{
    PinnedMigrationDirectory, optional_directory, require_allowed_membership, require_directory,
    require_empty, require_exact_membership, require_regular, required_directory,
};
use super::filesystem_migration_reader_fence;
use super::filesystem_migration_test_fixture::open_authority;
use super::{
    FilesystemMigrationRefusal as Refusal, StoreMigrationFixedStage, StoreMigrationStorage,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{
    FilesystemEntryIdentity, FilesystemEntryKind, FilesystemOperationRefusal, ReaderFenceRefusal,
};

fn payload<T: Error + Copy + 'static, R>(result: io::Result<R>) -> Option<(io::ErrorKind, T)> {
    let error = result.err()?;
    let refusal = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<T>())
        .copied()?;
    Some((error.kind(), refusal))
}

const fn invalid<T>(refusal: T) -> (io::ErrorKind, T) {
    (io::ErrorKind::InvalidData, refusal)
}

fn identity(path: &Path) -> io::Result<FilesystemEntryIdentity> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(FilesystemEntryIdentity::new(metadata.dev(), metadata.ino()))
}

#[test]
fn out_of_order_fixed_stage_phases_name_the_expected_and_active_stages()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("migration-refusal-stage-state")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;

    assert_eq!(
        payload(StoreMigrationStorage::synchronize_intent_stage(
            &mut authority
        )),
        Some(invalid(Refusal::StageInactive {
            expected: StoreMigrationFixedStage::Intent
        }))
    );
    StoreMigrationStorage::write_intent_stage(&mut authority, &intent)?;
    assert_eq!(
        payload(StoreMigrationStorage::write_intent_stage(
            &mut authority,
            &intent
        )),
        Some(invalid(Refusal::StageAlreadyActive {
            active: StoreMigrationFixedStage::Intent
        }))
    );
    assert_eq!(
        payload(StoreMigrationStorage::synchronize_marker_stage(
            &mut authority
        )),
        Some(invalid(Refusal::DifferentStageActive {
            expected: StoreMigrationFixedStage::Marker,
            observed: StoreMigrationFixedStage::Intent,
        }))
    );
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn linking_a_different_record_than_the_retained_stage_is_refused() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-refusal-stage-record")?;
    let root = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let intent = vec![0_u8; Artifact::Intent.encoded_length()];
    let marker = vec![0_u8; Artifact::Marker.encoded_length()];
    let stage = FilesystemMigrationFixedStage::create(&root, Artifact::Intent, &intent)?;

    assert_eq!(
        payload(stage.link(&root, Artifact::Marker, &marker)),
        Some(invalid(Refusal::StageRecordDisagreed {
            retained: StoreMigrationFixedStage::Intent,
            requested: StoreMigrationFixedStage::Marker,
        }))
    );
    drop(stage);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn migration_namespace_contradictions_report_typed_evidence() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-refusal-namespace")?;
    fs::write(sandbox.path().join("file"), b"x")?;
    fs::create_dir(sandbox.path().join("directory"))?;
    let root = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let kind = |expected, observed| Refusal::EntryKind { expected, observed };
    let (file, directory) = (FilesystemEntryKind::File, FilesystemEntryKind::Directory);

    assert_eq!(
        payload(require_directory(&root, "file")),
        Some(invalid(kind(directory, file)))
    );
    assert_eq!(
        payload(optional_directory(&root, "file")),
        Some(invalid(kind(directory, file)))
    );
    assert_eq!(
        payload(require_regular(&root, "directory", None)),
        Some(invalid(kind(file, directory)))
    );
    assert_eq!(
        payload(require_regular(&root, "file", Some(3))),
        Some(invalid(Refusal::FileLength {
            expected: 3,
            observed: 1
        }))
    );
    assert_eq!(
        payload(required_directory(&root, "absent")),
        Some(invalid(Refusal::DirectoryAbsent { name: "absent" }))
    );
    assert_eq!(
        payload(require_empty(&root)),
        Some(invalid(Refusal::NamespaceNotEmpty))
    );
    assert_eq!(
        payload(require_exact_membership(&root, &["file"])),
        Some(invalid(Refusal::MembershipDisagreed))
    );
    assert_eq!(
        payload(require_allowed_membership(&root, &["file"])),
        Some(invalid(Refusal::UnknownEntry))
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_replaced_pinned_migration_directory_reports_both_identities() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-refusal-directory-identity")?;
    let root = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let pinned = PinnedMigrationDirectory::admit(&root, "gc")?;
    let path = sandbox.path().join("gc");
    let expected = identity(&path)?;
    fs::rename(&path, sandbox.path().join("displaced"))?;
    fs::create_dir(&path)?;
    let observed = identity(&path)?;

    assert_eq!(
        payload(pinned.verify(&root)),
        Some(invalid(FilesystemOperationRefusal::IdentityChanged {
            expected,
            observed
        }))
    );
    drop(pinned);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_nonempty_migration_reader_fence_reports_both_lengths() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-refusal-reader-fence")?;
    let root = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    drop(filesystem_migration_reader_fence::create(&root)?);
    fs::write(sandbox.path().join("reader.lock"), b"abc")?;
    let fence = filesystem_migration_reader_fence::open(&root)?;

    assert_eq!(
        payload(filesystem_migration_reader_fence::verify(&root, &fence)),
        Some(invalid(ReaderFenceRefusal::Length {
            observed_handle: 3,
            observed_entry: 3
        }))
    );
    drop(fence);
    sandbox.remove()?;
    Ok(())
}
