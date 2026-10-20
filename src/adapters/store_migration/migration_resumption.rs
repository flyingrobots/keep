//! This boundary module owns resuming an interrupted migration at one exact
//! phase.

use std::io;

use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    StoreMigrationError, StoreMigrationPhase, StoreMigrationStorage,
};

/// The three canonical records one migration publishes.
pub(super) struct MigrationRecords<'a> {
    pub(super) intent: &'a CanonicalStoreMigrationIntent,
    pub(super) marker: CanonicalStoreFormatMarker,
    pub(super) receipt: CanonicalStoreMigrationReceipt,
}

impl<'a> MigrationRecords<'a> {
    pub(super) fn for_intent(intent: &'a CanonicalStoreMigrationIntent) -> Self {
        let marker = CanonicalStoreFormatMarker::version_two();
        let receipt = CanonicalStoreMigrationReceipt::from_canonical(intent, &marker);
        Self {
            intent,
            marker,
            receipt,
        }
    }
}

/// Resumes one migration at `from` and runs every later phase in order.
///
/// The storage must already hold the handles the phases before `from`
/// established; a recovery storage adopts them from the observed residue.
/// No current-state verification runs here, because recovery verified the
/// persisted intent before planning. The returned receipt exists only after
/// the final store-root synchronization.
///
/// # Errors
///
/// Returns [`StoreMigrationError::Storage`] naming the exact phase that
/// refused. Failure returns no receipt.
pub fn resume_store_migration(
    storage: &mut impl StoreMigrationStorage,
    intent: &CanonicalStoreMigrationIntent,
    from: StoreMigrationPhase,
) -> Result<CanonicalStoreMigrationReceipt, StoreMigrationError> {
    let records = MigrationRecords::for_intent(intent);
    let start = StoreMigrationPhase::ALL
        .iter()
        .position(|phase| *phase == from)
        .unwrap_or(StoreMigrationPhase::ALL.len());
    for phase in StoreMigrationPhase::ALL.iter().skip(start) {
        execute_phase(storage, *phase, &records)?;
    }
    Ok(records.receipt)
}

/// Runs exactly one phase against the storage.
pub(super) fn execute_phase(
    storage: &mut impl StoreMigrationStorage,
    phase: StoreMigrationPhase,
    records: &MigrationRecords<'_>,
) -> Result<(), StoreMigrationError> {
    let result = match phase {
        StoreMigrationPhase::WriteIntentStage
        | StoreMigrationPhase::SynchronizeIntentStage
        | StoreMigrationPhase::LinkIntent
        | StoreMigrationPhase::SynchronizeRootAfterIntent
        | StoreMigrationPhase::RemoveIntentStage
        | StoreMigrationPhase::SynchronizeRootAfterIntentCleanup
        | StoreMigrationPhase::AdmitReaderFence
        | StoreMigrationPhase::AdmitNamespacePrefix
        | StoreMigrationPhase::SynchronizeRootAfterNamespace => {
            intent_and_namespace_phase(storage, phase, records.intent)
        }
        StoreMigrationPhase::WriteMarkerStage
        | StoreMigrationPhase::SynchronizeMarkerStage
        | StoreMigrationPhase::LinkMarker
        | StoreMigrationPhase::SynchronizeRootAfterMarker
        | StoreMigrationPhase::RemoveMarkerStage
        | StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup => {
            marker_phase(storage, phase, &records.marker)
        }
        StoreMigrationPhase::WriteReceiptStage
        | StoreMigrationPhase::SynchronizeReceiptStage
        | StoreMigrationPhase::LinkReceipt
        | StoreMigrationPhase::SynchronizeRootAfterReceipt
        | StoreMigrationPhase::RemoveReceiptStage
        | StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup => {
            receipt_phase(storage, phase, &records.receipt)
        }
    };
    result.map_err(|source| StoreMigrationError::Storage { phase, source })
}

fn intent_and_namespace_phase(
    storage: &mut impl StoreMigrationStorage,
    phase: StoreMigrationPhase,
    intent: &CanonicalStoreMigrationIntent,
) -> io::Result<()> {
    match phase {
        StoreMigrationPhase::WriteIntentStage => storage.write_intent_stage(intent),
        StoreMigrationPhase::SynchronizeIntentStage => storage.synchronize_intent_stage(),
        StoreMigrationPhase::LinkIntent => storage.link_intent(intent),
        StoreMigrationPhase::SynchronizeRootAfterIntent => storage.synchronize_root_after_intent(),
        StoreMigrationPhase::RemoveIntentStage => storage.remove_intent_stage(),
        StoreMigrationPhase::SynchronizeRootAfterIntentCleanup => {
            storage.synchronize_root_after_intent_cleanup()
        }
        StoreMigrationPhase::AdmitReaderFence => storage.admit_reader_fence(),
        StoreMigrationPhase::AdmitNamespacePrefix => storage.admit_namespace_prefix(),
        StoreMigrationPhase::SynchronizeRootAfterNamespace => {
            storage.synchronize_root_after_namespace()
        }
        _ => Err(wrong_section(phase)),
    }
}

fn marker_phase(
    storage: &mut impl StoreMigrationStorage,
    phase: StoreMigrationPhase,
    marker: &CanonicalStoreFormatMarker,
) -> io::Result<()> {
    match phase {
        StoreMigrationPhase::WriteMarkerStage => storage.write_marker_stage(marker),
        StoreMigrationPhase::SynchronizeMarkerStage => storage.synchronize_marker_stage(),
        StoreMigrationPhase::LinkMarker => storage.link_marker(marker),
        StoreMigrationPhase::SynchronizeRootAfterMarker => storage.synchronize_root_after_marker(),
        StoreMigrationPhase::RemoveMarkerStage => storage.remove_marker_stage(),
        StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup => {
            storage.synchronize_root_after_marker_cleanup()
        }
        _ => Err(wrong_section(phase)),
    }
}

fn receipt_phase(
    storage: &mut impl StoreMigrationStorage,
    phase: StoreMigrationPhase,
    receipt: &CanonicalStoreMigrationReceipt,
) -> io::Result<()> {
    match phase {
        StoreMigrationPhase::WriteReceiptStage => storage.write_receipt_stage(receipt),
        StoreMigrationPhase::SynchronizeReceiptStage => storage.synchronize_receipt_stage(),
        StoreMigrationPhase::LinkReceipt => storage.link_receipt(receipt),
        StoreMigrationPhase::SynchronizeRootAfterReceipt => {
            storage.synchronize_root_after_receipt()
        }
        StoreMigrationPhase::RemoveReceiptStage => storage.remove_receipt_stage(),
        StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup => {
            storage.synchronize_root_after_receipt_cleanup()
        }
        _ => Err(wrong_section(phase)),
    }
}

fn wrong_section(phase: StoreMigrationPhase) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        super::FilesystemMigrationRefusal::WrongPhaseSection { observed: phase },
    )
}
