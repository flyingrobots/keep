//! Public platform-admission laws; medium Linux filesystem evidence for #150.
//! Oracle: KEEP-CATALOG-007 requires platform admission independently of lock ownership.

#![cfg(all(target_os = "linux", feature = "repository-tasks"))]

#[path = "catalog_platform_admission/unsupported_directory.rs"]
pub mod unsupported_directory;

use std::error::Error;
use std::fs;
use std::io::ErrorKind;

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
