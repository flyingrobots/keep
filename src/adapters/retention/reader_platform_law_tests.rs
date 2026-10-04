//! Reader platform admission at the public snapshot boundaries.
//!
//! Size: medium. Oracle: production readers require the existing writable,
//! case-sensitive local ext4 profile, independently of valid migration bytes.
//! The negative fixture deliberately uses test-only writer admission on tmpfs.
//! Delete when the supported reader profile changes or stronger public laws subsume it.

use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use super::filesystem_retention_test_fixture::{
    CATALOG_NAME, SEGMENT_NAME, fixture, migrated_store,
};
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, DurableSnapshot, DurableStoreError,
    FilesystemPlatformAdmission, FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError,
    FilesystemStoreMigrationAuthority, FilesystemWriterLock, ReaderAttemptLimit, SegmentReadPolicy,
    execute_store_migration,
};

#[test]
fn a_canonical_tmpfs_store_refuses_the_public_reader_profile() -> Result<(), Box<dyn Error>> {
    let store = TmpfsStore::create("direct")?;
    let refusal =
        FilesystemRetentionSnapshot::load(&store.0, policy()?, ReaderAttemptLimit::DEFAULT)
            .err()
            .ok_or("valid migration bytes admitted an unsupported reader filesystem")?;
    assert!(
        matches!(&refusal, FilesystemRetentionSnapshotError::Admission { source }
        if source.kind() == io::ErrorKind::Unsupported),
        "unsupported filesystem must refuse at admission: {refusal:?}"
    );
    Ok(())
}

#[test]
fn a_canonical_tmpfs_store_refuses_durable_snapshot_admission() -> Result<(), Box<dyn Error>> {
    let store = TmpfsStore::create("durable")?;
    let refusal = DurableSnapshot::open(&store.0, policy()?, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("durable snapshot accepted an unsupported filesystem")?;
    assert!(
        matches!(&refusal, DurableStoreError::Snapshot(source)
        if matches!(source.as_ref(), FilesystemRetentionSnapshotError::Admission { source }
            if source.kind() == io::ErrorKind::Unsupported)),
        "durable admission must preserve the unsupported platform boundary: {refusal:?}"
    );
    Ok(())
}

#[test]
fn reader_platform_admission_does_not_acquire_writer_authority() -> Result<(), Box<dyn Error>> {
    let store = migrated_store("reader-platform-existing-writer")?;
    let writer = FilesystemWriterLock::try_acquire(store.path())?;

    let snapshot =
        FilesystemRetentionSnapshot::load(store.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;

    assert_eq!(
        snapshot.catalog().generation().get(),
        1,
        "reader must admit its catalog while another authority holds the writer lock"
    );
    drop(writer);
    Ok(())
}

struct TmpfsStore(PathBuf);

impl TmpfsStore {
    // Atomically claim only a name we created; stale or concurrent fixtures
    // remain untouched. The finite retry budget bounds test setup work.
    fn reserve(name: &str) -> io::Result<Self> {
        for attempt in 0_u16..1_024 {
            let path = PathBuf::from("/dev/shm").join(format!(
                "keep-reader-profile-{name}-{}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(source),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "tmpfs fixture names exhausted",
        ))
    }

    fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let filesystem = rustix::fs::fstatfs(&fs::File::open("/dev/shm")?)?;
        assert_eq!(
            filesystem.f_type, 0x0102_1994,
            "negative profile requires Linux tmpfs"
        );
        let store = Self::reserve(name)?;
        let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(&store.0)?;
        for (name, encoded) in [
            (
                format!("segments/{SEGMENT_NAME}"),
                include_str!("../../../conformance/segment-store/v1/one-zero-bundle-segment.hex"),
            ),
            (
                format!("catalogs/{CATALOG_NAME}"),
                include_str!("../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex"),
            ),
            (
                "HEAD".into(),
                include_str!("../../../conformance/segment-store/v1/one-zero-bundle-head.hex"),
            ),
        ] {
            fs::write(store.0.join(name), fixture(encoded)?)?;
        }
        let mut authority =
            FilesystemStoreMigrationAuthority::open(admission, SegmentReadPolicy::MAXIMUM)?;
        let intent = authority.observe_intent()?;
        let _receipt = execute_store_migration(&mut authority, &intent)?;
        Ok(store)
    }
}

impl Drop for TmpfsStore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}
