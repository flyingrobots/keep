//! Laws: every root that is not a recoverable initialization prefix refuses, unchanged.

use std::fs;
use std::path::PathBuf;

use super::initialization_test_fixture::{
    TestResult, initialize, initialize_until, require_refusal, witness,
};
use super::{StoreInitializationDirectory as Directory, StoreInitializationRefusal as Refusal};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{CanonicalStoreFormatMarker, FORMAT_MARKER_LENGTH, FilesystemEntryKind};

/// Number of boundary notifications before `AdmitDirectory(Catalogs)` begins:
/// lock (2), marker write (2), stage sync (2), root sync (2), fence (2), and
/// the staging and segments directories (4).
const BEFORE_CATALOGS: usize = 14;

fn refuses_unchanged(
    name: &str,
    prepare: impl FnOnce(&std::path::Path) -> TestResult<()>,
    expected: &Refusal,
) -> TestResult<()> {
    let sandbox = TestDirectory::create(name)?;
    prepare(sandbox.path())?;
    let before = witness(sandbox.path())?;

    require_refusal(initialize(sandbox.path()), expected)?;

    assert_eq!(
        witness(sandbox.path())?,
        before,
        "{name}: refusal mutated the root"
    );
    Ok(())
}

#[test]
fn a_foreign_file_is_refused_before_writer_lock_creation() -> TestResult<()> {
    refuses_unchanged(
        "initialization-foreign-file",
        |root| Ok(fs::write(root.join("notes.txt"), b"mine")?),
        &Refusal::NotAPrefix {
            entry: PathBuf::from("notes.txt"),
        },
    )
}

#[test]
fn protocol_directories_without_a_marker_stage_are_not_adopted() -> TestResult<()> {
    refuses_unchanged(
        "initialization-directories-without-stage",
        |root| {
            fs::write(root.join("writer.lock"), b"")?;
            Ok(fs::create_dir(root.join("staging"))?)
        },
        &Refusal::NotAPrefix {
            entry: PathBuf::from("staging"),
        },
    )
}

#[test]
fn a_directory_out_of_creation_order_is_refused() -> TestResult<()> {
    refuses_unchanged(
        "initialization-out-of-order",
        |root| {
            let interrupted = initialize_until(root, BEFORE_CATALOGS);
            assert!(
                interrupted.is_err(),
                "the interruption must stop initialization"
            );
            fs::remove_dir(root.join(Directory::Segments.path()))?;
            Ok(fs::create_dir(root.join(Directory::Catalogs.path()))?)
        },
        &Refusal::NotAPrefix {
            entry: PathBuf::from("catalogs"),
        },
    )
}

#[test]
fn retention_content_inside_a_prefix_contradicts_empty_retention() -> TestResult<()> {
    refuses_unchanged(
        "initialization-stray-retention-root",
        |root| {
            let interrupted = initialize_until(root, BEFORE_CATALOGS);
            assert!(
                interrupted.is_err(),
                "the interruption must stop initialization"
            );
            for directory in [
                Directory::Catalogs,
                Directory::Retention,
                Directory::RetentionRoots,
            ] {
                fs::create_dir(root.join(directory.path()))?;
            }
            Ok(fs::write(root.join("retention/roots/stray"), b"root")?)
        },
        &Refusal::NotAPrefix {
            entry: PathBuf::from("retention/roots/stray"),
        },
    )
}

#[test]
fn a_marker_stage_that_is_not_a_canonical_prefix_is_refused() -> TestResult<()> {
    refuses_unchanged(
        "initialization-foreign-stage",
        |root| {
            fs::write(root.join("writer.lock"), b"")?;
            Ok(fs::write(root.join("FORMAT.next"), b"KEEP:NOT:A:MARKER")?)
        },
        &Refusal::MarkerStageMismatch {
            expected_length: FORMAT_MARKER_LENGTH,
            observed_length: 17,
        },
    )
}

#[test]
fn a_nonempty_writer_lock_is_refused() -> TestResult<()> {
    refuses_unchanged(
        "initialization-nonempty-lock",
        |root| Ok(fs::write(root.join("writer.lock"), b"x")?),
        &Refusal::NonEmptyLockFile {
            entry: "writer.lock",
            observed_length: 1,
        },
    )
}

#[cfg(unix)]
#[test]
fn a_linked_protocol_directory_is_refused_by_kind() -> TestResult<()> {
    refuses_unchanged(
        "initialization-linked-directory",
        |root| {
            let interrupted = initialize_until(root, BEFORE_CATALOGS);
            assert!(
                interrupted.is_err(),
                "the interruption must stop initialization"
            );
            fs::remove_dir(root.join("segments"))?;
            Ok(std::os::unix::fs::symlink(
                "staging",
                root.join("segments"),
            )?)
        },
        &Refusal::WrongKind {
            entry: "segments",
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::Symlink,
        },
    )
}

#[test]
fn a_committed_store_refuses_a_second_initialization() -> TestResult<()> {
    refuses_unchanged(
        "initialization-committed",
        |root| {
            drop(initialize(root)?);
            Ok(())
        },
        &Refusal::AlreadyInitialized,
    )
}

#[test]
fn a_marker_beside_its_stage_must_be_the_same_inode() -> TestResult<()> {
    refuses_unchanged(
        "initialization-marker-copy",
        |root| {
            drop(initialize(root)?);
            let marker = CanonicalStoreFormatMarker::version_two();
            Ok(fs::write(root.join("FORMAT.next"), marker.encoded())?)
        },
        &Refusal::MarkerLinkMismatch,
    )
}
