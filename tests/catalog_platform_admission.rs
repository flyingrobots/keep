//! Public platform-admission laws; medium Linux filesystem evidence for #150.
//! Oracle: KEEP-CATALOG-007 requires platform admission independently of lock ownership.

#![cfg(all(target_os = "linux", feature = "repository-tasks"))]

#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod admitted_directory;
#[path = "catalog_platform_admission/unsupported_directory.rs"]
pub mod unsupported_directory;

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::io::Write;

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemCatalogPublisher,
    FilesystemPlatformAdmission, FilesystemWriterLock, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordLimit, StoreInitializationError, StoreInitializationPhase,
};

#[test]
fn a_writer_lock_cannot_mint_publication_authority_on_a_refused_platform()
-> Result<(), Box<dyn Error>> {
    let directory = unsupported_directory::UnsupportedDirectory::create()?;
    let refusal = match FilesystemPlatformAdmission::initialize(directory.path()) {
        Ok(_admitted) => return Err("probe filesystem unexpectedly admitted".into()),
        Err(error) => error,
    };
    assert!(
        matches!(refusal, StoreInitializationError::Io { phase, ref source }
        if phase == StoreInitializationPhase::AdmitPlatform
        && source.kind() == ErrorKind::Unsupported)
    );
    fs::write(directory.path().join("writer.lock"), [])?;
    for name in ["staging", "segments", "catalogs"] {
        fs::create_dir(directory.path().join(name))?;
    }
    let lock = FilesystemWriterLock::try_acquire(directory.path())?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let error = match FilesystemCatalogPublisher::open_unchecked_for_repository_tasks(lock, policy)
    {
        Ok(_publisher) => return Err("refused platform acquired public publisher authority".into()),
        Err(error) => error,
    };
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    directory.remove()?;
    Ok(())
}

#[test]
fn repository_publisher_preserves_staging_on_an_admitted_platform() -> Result<(), Box<dyn Error>> {
    let directory = admitted_directory::TestDirectory::create("catalog-admitted-platform")?;
    drop(FilesystemPlatformAdmission::initialize(directory.path())?);
    let lock = FilesystemWriterLock::try_acquire(directory.path())?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let publisher = FilesystemCatalogPublisher::open_unchecked_for_repository_tasks(lock, policy)?;
    let mut stage = publisher.create_segment_stage()?;
    stage.write_all(b"retained stage evidence")?;
    drop(stage);
    drop(publisher);
    assert_eq!(
        fs::read(directory.path().join("staging/current.seg"))?,
        b"retained stage evidence"
    );
    directory.remove()?;
    Ok(())
}
