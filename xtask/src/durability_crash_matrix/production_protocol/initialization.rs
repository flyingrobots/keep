//! This module owns execution of the production initialization protocol.

use std::path::Path;

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemCatalogPublisher, LayoutEntryLimit,
    RepositoryInitializationStorage, SegmentReadPolicy, SegmentRecordLimit, initialize_store,
};

use super::control::CrashControl;
use super::initialization_storage::CrashInitializationStorage;
use super::{DurabilityCrashMatrixError, verification};

const RESTART_BYTE_LIMIT: u64 = 1_048_576;

pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    let storage = RepositoryInitializationStorage::admit_unchecked(store_root)
        .map_err(|source| DurabilityCrashMatrixError::io("open initialization storage", source))?;
    let mut storage = CrashInitializationStorage::new(storage, control);
    initialize_store(&mut storage)
        .map(|_receipt| ())
        .map_err(|source| verification("execute production store initialization", source))
}

pub(super) fn publisher(
    store_root: &Path,
) -> Result<FilesystemCatalogPublisher, DurabilityCrashMatrixError> {
    let mut storage = RepositoryInitializationStorage::admit_unchecked(store_root)
        .map_err(|source| DurabilityCrashMatrixError::io("open initialization storage", source))?;
    let _receipt = initialize_store(&mut storage)
        .map_err(|source| verification("initialize production crash store", source))?;
    let lock = storage.into_writer_lock().map_err(|source| {
        DurabilityCrashMatrixError::io("retain initialized writer lock", source)
    })?;
    FilesystemCatalogPublisher::open_unchecked_for_repository_tasks(lock, restart_policy()?)
        .map_err(|source| DurabilityCrashMatrixError::io("open crash catalog publisher", source))
}

pub(super) fn restart_policy() -> Result<CatalogRestartPolicy, DurabilityCrashMatrixError> {
    let byte_limit = CatalogRestartByteLimit::new(RESTART_BYTE_LIMIT)
        .map_err(|source| verification("construct crash restart byte limit", source))?;
    Ok(CatalogRestartPolicy::new(segment_policy(), byte_limit))
}

pub(super) const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
