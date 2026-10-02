//! These laws own reader admission against migration-bound physical root identity.

use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;

use super::filesystem_retention_test_fixture::migrated_store;
use super::{FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, ReaderAttemptLimit};
use crate::adapters::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemPlatformAdmissionError,
    SegmentReadPolicy, StoreRootIdentityCoordinate,
};

// Size: medium. Oracle: a reader must admit the root named by migration.intent.
// Delete only with migration binding or a stronger public reader-boundary law.
#[test]
fn a_reader_refuses_migration_records_bound_to_another_root() -> Result<(), Box<dyn Error>> {
    let donor = migrated_store("reader-migration-identity-donor")?;
    let recipient = migrated_store("reader-migration-identity-recipient")?;
    for name in ["FORMAT", "migration.intent", "migration.receipt"] {
        fs::copy(donor.path().join(name), recipient.path().join(name))?;
    }
    let expected = fs::metadata(donor.path())?.ino();
    let observed = fs::metadata(recipient.path())?.ino();
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );

    let result =
        FilesystemRetentionSnapshot::load(recipient.path(), policy, ReaderAttemptLimit::DEFAULT);

    let error = match result {
        Err(FilesystemRetentionSnapshotError::Admission { source }) => source,
        Ok(_) => {
            return Err("reader accepted migration records naming another physical root".into());
        }
        Err(other) => {
            return Err(format!("reader root identity must refuse at admission: {other:?}").into());
        }
    };
    let refusal = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemPlatformAdmissionError>());
    assert!(
        matches!(refusal, Some(FilesystemPlatformAdmissionError::RootIdentityChanged {
        coordinate: StoreRootIdentityCoordinate::File,
        expected: actual_expected, observed: actual_observed,
    }) if *actual_expected == expected && *actual_observed == observed),
        "reader must preserve exact bound inode {expected} and observed inode {observed}: {error:?}"
    );
    Ok(())
}
