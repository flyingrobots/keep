//! This boundary module owns ordered version-2 store-migration execution.

use std::io;

use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    StoreMigrationError, StoreMigrationPhase, StoreMigrationStorage,
};

/// Executes one version-2 migration under revalidated version-1 authority.
///
/// The returned receipt exists only after all canonical artifacts are visible,
/// all retained stages are removed, and final store-root cleanup is synchronized.
///
/// # Errors
///
/// Returns [`StoreMigrationError`] for current-state revalidation or the exact
/// failed durability phase. Failure returns no receipt.
pub fn execute_store_migration(
    storage: &mut impl StoreMigrationStorage,
    intent: &CanonicalStoreMigrationIntent,
) -> Result<CanonicalStoreMigrationReceipt, StoreMigrationError> {
    storage
        .verify_current(intent)
        .map_err(|source| StoreMigrationError::CurrentVerification { source })?;
    let marker = CanonicalStoreFormatMarker::version_two();
    let receipt = CanonicalStoreMigrationReceipt::from_canonical(intent, &marker);
    execute_intent(storage, intent)?;
    execute_namespace(storage)?;
    execute_marker(storage, &marker)?;
    execute_receipt(storage, &receipt)?;
    Ok(receipt)
}

fn execute_intent(
    storage: &mut impl StoreMigrationStorage,
    intent: &CanonicalStoreMigrationIntent,
) -> Result<(), StoreMigrationError> {
    require(
        storage.write_intent_stage(intent),
        StoreMigrationPhase::WriteIntentStage,
    )?;
    require(
        storage.synchronize_intent_stage(),
        StoreMigrationPhase::SynchronizeIntentStage,
    )?;
    require(storage.link_intent(intent), StoreMigrationPhase::LinkIntent)?;
    require(
        storage.synchronize_root_after_intent(),
        StoreMigrationPhase::SynchronizeRootAfterIntent,
    )?;
    require(
        storage.remove_intent_stage(),
        StoreMigrationPhase::RemoveIntentStage,
    )?;
    require(
        storage.synchronize_root_after_intent_cleanup(),
        StoreMigrationPhase::SynchronizeRootAfterIntentCleanup,
    )
}

fn execute_namespace(storage: &mut impl StoreMigrationStorage) -> Result<(), StoreMigrationError> {
    require(
        storage.admit_reader_fence(),
        StoreMigrationPhase::AdmitReaderFence,
    )?;
    require(
        storage.admit_namespace_prefix(),
        StoreMigrationPhase::AdmitNamespacePrefix,
    )?;
    require(
        storage.synchronize_root_after_namespace(),
        StoreMigrationPhase::SynchronizeRootAfterNamespace,
    )
}

fn execute_marker(
    storage: &mut impl StoreMigrationStorage,
    marker: &CanonicalStoreFormatMarker,
) -> Result<(), StoreMigrationError> {
    require(
        storage.write_marker_stage(marker),
        StoreMigrationPhase::WriteMarkerStage,
    )?;
    require(
        storage.synchronize_marker_stage(),
        StoreMigrationPhase::SynchronizeMarkerStage,
    )?;
    require(storage.link_marker(marker), StoreMigrationPhase::LinkMarker)?;
    require(
        storage.synchronize_root_after_marker(),
        StoreMigrationPhase::SynchronizeRootAfterMarker,
    )?;
    require(
        storage.remove_marker_stage(),
        StoreMigrationPhase::RemoveMarkerStage,
    )?;
    require(
        storage.synchronize_root_after_marker_cleanup(),
        StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup,
    )
}

fn execute_receipt(
    storage: &mut impl StoreMigrationStorage,
    receipt: &CanonicalStoreMigrationReceipt,
) -> Result<(), StoreMigrationError> {
    require(
        storage.write_receipt_stage(receipt),
        StoreMigrationPhase::WriteReceiptStage,
    )?;
    require(
        storage.synchronize_receipt_stage(),
        StoreMigrationPhase::SynchronizeReceiptStage,
    )?;
    require(
        storage.link_receipt(receipt),
        StoreMigrationPhase::LinkReceipt,
    )?;
    require(
        storage.synchronize_root_after_receipt(),
        StoreMigrationPhase::SynchronizeRootAfterReceipt,
    )?;
    require(
        storage.remove_receipt_stage(),
        StoreMigrationPhase::RemoveReceiptStage,
    )?;
    require(
        storage.synchronize_root_after_receipt_cleanup(),
        StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup,
    )
}

fn require<T>(result: io::Result<T>, phase: StoreMigrationPhase) -> Result<T, StoreMigrationError> {
    result.map_err(|source| StoreMigrationError::Storage { phase, source })
}
