//! Selected namespace entry kinds remain evidenced corruption, not raw I/O.
//!
//! Size: medium. Oracle: a selected namespace is a real directory, independently
//! of symlink targets; the original selected root bytes survive refusal.
//! Delete if stronger public selected-namespace coverage subsumes these cases.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, root_pool_path,
};
use crate::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemEntryKind,
    FilesystemNamespaceRefusal, FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError,
    ReaderAttemptLimit, SegmentReadPolicy, VerificationDepth, VerificationError,
    VerificationObservation, VerificationRefusal, VerificationSource, VerificationSubject,
    execute_retention_publication,
};
use std::{error::Error, fs, os::unix::fs::symlink};

#[test]
fn a_selected_namespace_file_is_typed_corruption() -> Result<(), Box<dyn Error>> {
    assert_kind("verification-namespace-file", FilesystemEntryKind::File)
}

#[test]
fn a_selected_namespace_symlink_is_typed_corruption() -> Result<(), Box<dyn Error>> {
    assert_kind(
        "verification-namespace-symlink",
        FilesystemEntryKind::Symlink,
    )
}

fn assert_kind(name: &str, observed: FilesystemEntryKind) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let bytes = fixture(ROOT_HEX)?;
    let _receipt = execute_retention_publication(&mut authority, &initial_preparation(&bytes)?)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let path = root_pool_path(sandbox.path(), &root);
    let parent = path.parent().ok_or("root parent absent")?;
    let retained = parent.with_extension("original");
    let view = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        CatalogRestartPolicy::new(
            SegmentReadPolicy::MAXIMUM,
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    )?;
    fs::rename(parent, &retained)?;
    match observed {
        FilesystemEntryKind::File => fs::write(parent, b"preserve namespace file")?,
        FilesystemEntryKind::Symlink => symlink(&retained, parent)?,
        _ => return Err("unsupported test entry kind".into()),
    }
    let namespace = root.root().namespace().digest();
    let error = view
        .verify_retention(namespace, VerificationDepth::Checksum)
        .err()
        .ok_or("wrong-kind namespace certified")?;
    let expected_cause = FilesystemNamespaceRefusal::WrongKind {
        expected: FilesystemEntryKind::Directory,
        observed,
    };
    assert!(
        matches!(&error, VerificationError::Refused { refusal: VerificationRefusal::Corrupt {
        subject: VerificationSubject::RetentionNamespace { namespace: actual },
        expected: VerificationObservation::Canonical, observed: VerificationObservation::Refused },
        source: Some(source) } if *actual == namespace && matches!(source.as_ref(),
        VerificationSource::Retention(FilesystemRetentionSnapshotError::Root { source })
        if source.get_ref().and_then(|cause| cause.downcast_ref::<FilesystemNamespaceRefusal>()) == Some(&expected_cause))),
        "selected namespace must preserve exact kind corruption: {error:?}"
    );
    assert_eq!(
        fs::read(retained.join(path.file_name().ok_or("root name absent")?))?,
        bytes
    );
    assert_eq!(
        fs::symlink_metadata(parent)?.file_type().is_symlink(),
        observed == FilesystemEntryKind::Symlink
    );
    Ok(())
}
