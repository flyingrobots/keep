//! Reader admission on every admitted platform leaves writer authority alone.
//!
//! Size: medium. Oracle: a reader admits its catalog while another authority
//! holds the writer lock. Delete when stronger public reader laws subsume it.

use std::error::Error;

use super::filesystem_retention_test_fixture::migrated_store;
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
    FilesystemWriterLock, ReaderAttemptLimit, SegmentReadPolicy,
};

#[test]
fn reader_platform_admission_does_not_acquire_writer_authority() -> Result<(), Box<dyn Error>> {
    let store = migrated_store("reader-platform-existing-writer")?;
    let writer = FilesystemWriterLock::try_acquire(store.path())?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );

    let snapshot =
        FilesystemRetentionSnapshot::load(store.path(), policy, ReaderAttemptLimit::DEFAULT)?;

    assert_eq!(
        snapshot.catalog().generation().get(),
        1,
        "reader must admit its catalog while another authority holds the writer lock"
    );
    drop(writer);
    Ok(())
}
