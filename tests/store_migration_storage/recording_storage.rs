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
}

impl RecordingStorage {
    /// Returns attempted migration phases in call order.
    pub fn observed(&self) -> &[StoreMigrationPhase] {
        &self.observed
    }

    /// Returns the number of current-state verification attempts.
    pub const fn verification_count(&self) -> usize {
        self.verification_count
    }

    fn record(&mut self, phase: StoreMigrationPhase) {
        self.observed.push(phase);
    }
}

impl StoreMigrationStorage for RecordingStorage {
    fn verify_current(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.verification_count = self
            .verification_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("verification count overflow"))?;
        Ok(())
    }

    fn write_intent_stage(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteIntentStage);
        Ok(())
    }

    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeIntentStage);
        Ok(())
    }

    fn link_intent(&mut self, _intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkIntent);
        Ok(())
    }

    fn synchronize_root_after_intent(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterIntent);
        Ok(())
    }

    fn remove_intent_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveIntentStage);
        Ok(())
    }

    fn synchronize_root_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterIntentCleanup);
        Ok(())
    }

    fn admit_reader_fence(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::AdmitReaderFence);
        Ok(())
    }

    fn admit_namespace_prefix(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::AdmitNamespacePrefix);
        Ok(())
    }

    fn synchronize_root_after_namespace(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterNamespace);
        Ok(())
    }

    fn write_marker_stage(&mut self, _marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteMarkerStage);
        Ok(())
    }

    fn synchronize_marker_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeMarkerStage);
        Ok(())
    }

    fn link_marker(&mut self, _marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkMarker);
        Ok(())
    }

    fn synchronize_root_after_marker(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterMarker);
        Ok(())
    }

    fn remove_marker_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveMarkerStage);
        Ok(())
    }

    fn synchronize_root_after_marker_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup);
        Ok(())
    }

    fn write_receipt_stage(&mut self, _receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        self.record(StoreMigrationPhase::WriteReceiptStage);
        Ok(())
    }

    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeReceiptStage);
        Ok(())
    }

    fn link_receipt(&mut self, _receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        self.record(StoreMigrationPhase::LinkReceipt);
        Ok(())
    }

    fn synchronize_root_after_receipt(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterReceipt);
        Ok(())
    }

    fn remove_receipt_stage(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::RemoveReceiptStage);
        Ok(())
    }

    fn synchronize_root_after_receipt_cleanup(&mut self) -> io::Result<()> {
        self.record(StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup);
        Ok(())
    }
}
