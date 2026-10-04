//! A selected root must be a regular file, independently of symlink targets.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, root_pool_path,
};
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use crate::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy,
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, ReaderAttemptLimit,
    SegmentReadPolicy, VerificationDepth, VerificationError, VerificationObservation,
    VerificationRefusal, VerificationSource, VerificationSubject, execute_retention_publication,
};
use std::{error::Error, fs, os::unix::fs::symlink};

// Size: medium. Oracle: the selected pathname must itself be a regular root record.
// Delete if stronger public wrong-kind coverage subsumes this valid-target symlink.
#[test]
fn a_selected_root_symlink_is_typed_corruption() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verification-root-symlink")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let path = root_pool_path(sandbox.path(), &root);
    let target = path.with_extension("original");
    fs::rename(&path, &target)?;
    symlink(&target, &path)?;
    let view = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        CatalogRestartPolicy::new(
            SegmentReadPolicy::MAXIMUM,
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    )?;
    let namespace = root.root().namespace().digest();
    let error = view
        .verify_retention(namespace, VerificationDepth::Checksum)
        .err()
        .ok_or("symlink certified")?;
    let expected = VerificationRefusal::Corrupt {
        subject: VerificationSubject::RetentionNamespace { namespace },
        expected: VerificationObservation::Canonical,
        observed: VerificationObservation::Refused,
    };
    let actual = match &error {
        VerificationError::Refused {
            refusal,
            source: Some(source),
        } if *refusal == expected => match source.as_ref() {
            VerificationSource::Retention(FilesystemRetentionSnapshotError::Root { source }) => {
                source
                    .get_ref()
                    .and_then(|source| source.downcast_ref::<ExactRecordError>())
            }
            _ => None,
        },
        _ => None,
    };
    assert!(
        matches!(
            actual,
            Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
        ),
        "selected symlink must retain typed kind corruption: {error:?}"
    );
    Ok(())
}
