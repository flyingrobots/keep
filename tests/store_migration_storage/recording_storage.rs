//! This module owns the store-migration storage test double.

use std::io;

use keep::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    StoreMigrationPhase, StoreMigrationStorage,
};

#[derive(Default)]
/// Storage port that records every attempted migration phase.
pub struct RecordingStorage {
    observed: Vec<StoreMigrationPhase>,
    verification_count: usize,
    fail_at: Option<StoreMigrationPhase>,
    verification_failure: Option<io::ErrorKind>,
}

impl RecordingStorage {
    /// Creates storage that refuses at one exact migration phase.
    pub fn failing_at(phase: StoreMigrationPhase) -> Self {
        Self {
            fail_at: Some(phase),
            ..Self::default()
        }
    }

    /// Creates storage that refuses current-state verification.
    pub fn verification_failure() -> Self {
        Self {
            verification_failure: Some(io::ErrorKind::PermissionDenied),
            ..Self::default()
        }
    }

    /// Returns attempted migration phases in call order.
    pub fn observed(&self) -> &[StoreMigrationPhase] {
        &self.observed
    }

    /// Returns the number of current-state verification attempts.
    pub const fn verification_count(&self) -> usize {
        self.verification_count
    }

    fn record(&mut self, phase: StoreMigrationPhase) -> io::Result<()> {
        self.observed.push(phase);
        if self.fail_at == Some(phase) {
            Err(io::Error::other("injected store-migration failure"))
        } else {
            Ok(())
        }
    }
}

impl StoreMigrationStorage for RecordingStorage {
    fn verify_current(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.verification_count = self
            .verification_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("verification count overflow"))?;
        self.verification_failure
            .map_or(Ok(()), |kind| Err(kind.into()))
    }

    fn write_intent_stage(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteIntentStage)
    }

    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeIntentStage)
    }

    fn link_intent(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkIntent)
    }

    fn synchronize_root_after_intent(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterIntent)
    }

    fn remove_intent_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveIntentStage)
    }

    fn synchronize_root_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterIntentCleanup)
    }

    fn admit_reader_fence(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::AdmitReaderFence)
    }

    fn admit_namespace_prefix(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::AdmitNamespacePrefix)
    }

    fn synchronize_root_after_namespace(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterNamespace)
    }

    fn write_marker_stage(&mut self, _marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteMarkerStage)
    }

    fn synchronize_marker_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeMarkerStage)
    }

    fn link_marker(&mut self, _marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkMarker)
    }

    fn synchronize_root_after_marker(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterMarker)
    }

    fn remove_marker_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveMarkerStage)
    }

    fn synchronize_root_after_marker_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup)
    }

    fn write_receipt_stage(&mut self, _receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteReceiptStage)
    }

    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeReceiptStage)
    }

    fn link_receipt(&mut self, _receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkReceipt)
    }

    fn synchronize_root_after_receipt(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterReceipt)
    }

    fn remove_receipt_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveReceiptStage)
    }

    fn synchronize_root_after_receipt_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup)
    }
}
