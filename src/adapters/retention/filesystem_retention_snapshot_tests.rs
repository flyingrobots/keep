//! Reader fence and fenced snapshot laws over migrated stores.

use std::error::Error;
use std::fs;

use cap_std::fs::Dir;
use rustix::fs::{FlockOperation, flock};
use rustix::io::Errno;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, migrated_store, open_authority,
};
use super::{
    AdmittedRetentionRoot, FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError,
    ReaderAttemptLimit, ReaderFence, RetentionRootDecodeError,
};
use crate::adapters::{
    CatalogRestartByteLimit, CatalogRestartPolicy, SegmentReadPolicy, SegmentRecordLimit,
};
use crate::{LayoutEntryLimit, execute_retention_publication};

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}

#[test]
fn a_migrated_store_snapshot_binds_the_catalog_and_no_retention_head() -> Result<(), Box<dyn Error>>
{
    let sandbox = migrated_store("filesystem-retention-snapshot-empty")?;
    let snapshot =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    assert_eq!(snapshot.catalog().generation().get(), 1);
    assert!(snapshot.retention_head().is_none());
    assert!(snapshot.manifest().is_none());
    Ok(())
}

#[test]
fn a_published_generation_is_read_and_its_root_verified() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-snapshot-published")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let namespace = candidate.root().namespace().digest();

    let snapshot =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;

    let head = snapshot
        .retention_head()
        .ok_or("no retention head in the view")?;
    assert_eq!(head.generation(), preparation.liveness_generation());
    let root = snapshot
        .retained_root(namespace)?
        .ok_or("the manifest does not select the published namespace")?;
    assert_eq!(&*root, root_bytes.as_slice());
    Ok(())
}

// Size: medium. Oracle: checksum damage reports the exact checksum mismatch against the golden bytes.
// Delete when selected-root reads are removed or stronger corruption evidence subsumes it.
#[test]
fn root_checksum_damage_reports_its_exact_mismatch() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-snapshot-substituted")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let path = super::filesystem_retention_test_fixture::root_pool_path(sandbox.path(), &candidate);
    let mut corrupt = root_bytes.clone();
    *corrupt.last_mut().ok_or("empty root fixture")? ^= 0x01;
    let checksum_offset = root_bytes.len().checked_sub(32).ok_or("missing checksum")?;
    let expected_checksum: [u8; 32] = root_bytes
        .get(checksum_offset..)
        .ok_or("missing checksum")?
        .try_into()?;
    let observed_checksum: [u8; 32] = corrupt
        .get(checksum_offset..)
        .ok_or("missing damaged checksum")?
        .try_into()?;
    fs::write(&path, &corrupt)?;

    let snapshot =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    let error = snapshot
        .retained_root(candidate.root().namespace().digest())
        .err()
        .ok_or("a corrupt root pool entry was returned")?;

    assert!(
        matches!(
            error,
            FilesystemRetentionSnapshotError::Root { ref source }
                if source.kind() == std::io::ErrorKind::InvalidData
                    && matches!(source.get_ref().and_then(|cause| cause.downcast_ref::<RetentionRootDecodeError>()),
                        Some(RetentionRootDecodeError::ChecksumMismatch { expected, observed })
                            if *expected == expected_checksum && *observed == observed_checksum)
        ),
        "checksum damage must retain its exact decoder cause and checksum coordinates: {error:?}"
    );
    Ok(())
}

#[test]
// Size: medium. Oracle: live shared fences exclude a collector with WOULDBLOCK.
// Delete when reader fencing is removed or stronger lifetime evidence subsumes this law.
fn readers_share_the_fence_and_collection_cannot_take_it_exclusively() -> Result<(), Box<dyn Error>>
{
    let sandbox = migrated_store("filesystem-retention-snapshot-fence")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let first = ReaderFence::acquire(&root)?;
    let second = ReaderFence::acquire(&root)?;
    let collector = std::fs::File::open(sandbox.path().join("reader.lock"))?;

    let refused = flock(&collector, FlockOperation::NonBlockingLockExclusive);

    assert_eq!(
        refused,
        Err(Errno::WOULDBLOCK),
        "an exclusive fence must wait for readers"
    );
    drop(second);
    drop(first);
    flock(&collector, FlockOperation::NonBlockingLockExclusive)?;
    Ok(())
}

#[test]
// Size: medium. Oracle: the reader.lock protocol requires an empty regular file.
// Delete when that protocol is removed or a stronger public-reader law subsumes this case.
fn a_non_empty_reader_lock_refuses_the_fence() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("filesystem-retention-snapshot-bad-fence")?;
    fs::write(sandbox.path().join("reader.lock"), b"not empty")?;
    let error =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)
            .err()
            .ok_or("a non-empty reader.lock was accepted as the fence")?;
    let FilesystemRetentionSnapshotError::Fence { source } = error else {
        return Err("wrong fence refusal boundary".into());
    };
    assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        source
            .get_ref()
            .and_then(|source| source.downcast_ref::<crate::ReaderFenceRefusal>()),
        Some(&crate::ReaderFenceRefusal::Length {
            expected: 0,
            observed_handle: 9,
            observed_entry: 9,
        })
    );
    Ok(())
}
