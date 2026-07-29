//! This module owns ordered segment-store namespace initialization.

use std::io;

use super::{
    store_initialization_error::StoreInitializationError,
    store_initialization_phase::StoreInitializationPhase,
    store_initialization_receipt::StoreInitializationReceipt,
    store_initialization_storage::StoreInitializationStorage,
};

/// Executes the ordered segment-store initialization protocol.
///
/// The storage port must admit its platform before any namespace mutation,
/// retain writer authority after opening the lock file, and make every
/// directory operation idempotent by admitting an exact existing directory.
/// The function returns a receipt only after root synchronization succeeds.
/// It is synchronous, performs no internal heap allocation, invokes each port
/// operation at most once, and may block only inside the supplied storage
/// operations. Failure performs no implicit cleanup.
///
/// # Errors
///
/// Returns [`StoreInitializationError::Io`] at the first failed phase and does
/// not execute any later phase.
pub fn initialize_store(
    storage: &mut impl StoreInitializationStorage,
) -> Result<StoreInitializationReceipt, StoreInitializationError> {
    phase(
        StoreInitializationPhase::AdmitPlatform,
        storage.admit_platform(),
    )?;
    phase(
        StoreInitializationPhase::OpenAndLockWriterFile,
        storage.open_and_lock_writer_file(),
    )?;
    admit_directories(storage)?;
    phase(
        StoreInitializationPhase::SynchronizeRoot,
        storage.synchronize_root(),
    )?;
    Ok(StoreInitializationReceipt::new())
}

fn admit_directories(
    storage: &mut impl StoreInitializationStorage,
) -> Result<(), StoreInitializationError> {
    phase(
        StoreInitializationPhase::AdmitStagingDirectory,
        storage.admit_staging_directory(),
    )?;
    phase(
        StoreInitializationPhase::AdmitSegmentPoolDirectory,
        storage.admit_segment_pool_directory(),
    )?;
    phase(
        StoreInitializationPhase::AdmitCatalogPoolDirectory,
        storage.admit_catalog_pool_directory(),
    )?;
    Ok(())
}

fn phase(
    phase: StoreInitializationPhase,
    result: io::Result<()>,
) -> Result<(), StoreInitializationError> {
    result.map_err(|source| StoreInitializationError::Io { phase, source })
}
