//! Demonstrated namespace contradictions are distinct from failed observations.
//!
//! Size: medium. Oracle: canonical protocol membership and no-follow entry kinds.
//! Delete if stronger public namespace laws subsume these exact boundaries.

use super::filesystem_retention_test_fixture::migrated_store;
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemEntryKind, FilesystemNamespaceRefusal,
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, ReaderAttemptLimit,
    SegmentReadPolicy, VerificationError, VerificationObservation, VerificationRefusal,
    VerificationSource, VerificationSubject,
};
use std::{error::Error, fs, os::unix::fs::symlink, path::Path};

type ResultOf<T> = Result<T, Box<dyn Error>>;

#[test]
fn unexpected_store_entries_are_published_view_corruption() -> ResultOf<()> {
    let store = migrated_store("verification-unexpected-entry")?;
    let path = store.path().join("unexpected-entry");
    fs::write(&path, b"preserve unexpected evidence")?;
    assert_corrupt(store.path(), FilesystemNamespaceRefusal::UnexpectedEntry)?;
    assert_eq!(fs::read(path)?, b"preserve unexpected evidence");
    Ok(())
}

#[test]
fn required_directory_files_are_published_view_corruption() -> ResultOf<()> {
    let store = migrated_store("verification-required-directory-file")?;
    let path = store.path().join("gc");
    fs::remove_dir(&path)?;
    fs::write(&path, b"preserve wrong kind")?;
    assert_corrupt(
        store.path(),
        FilesystemNamespaceRefusal::WrongKind {
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::File,
        },
    )?;
    assert_eq!(fs::read(path)?, b"preserve wrong kind");
    Ok(())
}

#[test]
fn required_directory_symlinks_are_published_view_corruption() -> ResultOf<()> {
    let store = migrated_store("verification-required-directory-symlink")?;
    let path = store.path().join("gc");
    let target = store.path().join("retention");
    fs::remove_dir(&path)?;
    symlink(&target, &path)?;
    assert_corrupt(
        store.path(),
        FilesystemNamespaceRefusal::WrongKind {
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::Symlink,
        },
    )?;
    assert_eq!(fs::read_link(path)?, target);
    Ok(())
}

fn assert_corrupt(path: &Path, expected_cause: FilesystemNamespaceRefusal) -> ResultOf<()> {
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let error = FilesystemRetentionSnapshot::load_for_verification(
        path,
        policy,
        ReaderAttemptLimit::DEFAULT,
    )
    .err()
    .ok_or("noncanonical namespace admitted")?;
    assert!(
        matches!(&error, VerificationError::Refused { refusal: VerificationRefusal::Corrupt {
        subject: VerificationSubject::PublishedView, expected: VerificationObservation::Canonical,
        observed: VerificationObservation::Refused }, source: Some(source) } if matches!(source.as_ref(), VerificationSource::Retention(FilesystemRetentionSnapshotError::Admission { source }) if source.get_ref().and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>()) == Some(&expected_cause))),
        "demonstrated namespace contradiction must remain corrupt: {error:?}"
    );
    Ok(())
}
