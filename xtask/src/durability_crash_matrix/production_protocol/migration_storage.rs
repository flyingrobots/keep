//! This module owns crash injection around production store migration.

use std::io;

use keep::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    FilesystemStoreMigrationAuthority, StoreMigrationFixedStage, StoreMigrationStorage,
};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

// Canonical record types own fixed-width arrays of at least 96 bytes. Halving
// their width leaves a nonempty strict prefix without duplicating format sizes.
pub(super) struct CrashMigrationStorage<'control> {
    inner: FilesystemStoreMigrationAuthority,
    control: &'control mut CrashControl,
}

impl<'control> CrashMigrationStorage<'control> {
    pub(super) const fn new(
        inner: FilesystemStoreMigrationAuthority,
        control: &'control mut CrashControl,
    ) -> Self {
        Self { inner, control }
    }
}

impl StoreMigrationStorage for CrashMigrationStorage<'_> {
    fn verify_current(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        StoreMigrationStorage::verify_current(&mut self.inner, intent)
    }

    fn write_intent_stage(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        execute_write(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationWriteIntentStage,
            |inner| inner.write_intent_stage(intent),
            |inner| {
                inner.write_fixed_stage_prefix_for_repository_tasks(
                    StoreMigrationFixedStage::Intent,
                    intent.encoded(),
                    intent.encoded().len() / 2,
                )
            },
        )
    }

    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeIntentStage,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_intent_stage,
        )
    }

    fn link_intent(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationLinkIntent,
            DuringTiming::After,
            |inner| inner.link_intent(intent),
        )
    }

    fn synchronize_root_after_intent(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterIntent,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_intent,
        )
    }

    fn remove_intent_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationRemoveIntentStage,
            DuringTiming::After,
            FilesystemStoreMigrationAuthority::remove_intent_stage,
        )
    }

    fn synchronize_root_after_intent_cleanup(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterIntentCleanup,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_intent_cleanup,
        )
    }

    fn admit_reader_fence(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationAdmitReaderFence,
            DuringTiming::After,
            FilesystemStoreMigrationAuthority::admit_reader_fence,
        )
    }

    fn admit_namespace_prefix(&mut self) -> io::Result<()> {
        let point = DurabilityCrashPoint::MigrationAdmitNamespacePrefix;
        match self.control.position(point) {
            None => self.inner.admit_namespace_prefix(),
            Some(DurabilityCrashPosition::Before) => self.control.await_process_death(),
            Some(DurabilityCrashPosition::During) => {
                let reached = self
                    .control
                    .occurrence()
                    .map_or(1, |occurrence| occurrence.get().saturating_add(1));
                let reached = usize::try_from(reached).map_err(io::Error::other)?;
                self.inner
                    .admit_namespace_prefix_for_repository_tasks(reached)?;
                self.control.await_process_death()
            }
            Some(DurabilityCrashPosition::After) => {
                self.inner.admit_namespace_prefix()?;
                self.control.await_process_death()
            }
        }
    }

    fn synchronize_root_after_namespace(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterNamespace,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_namespace,
        )
    }

    fn write_marker_stage(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        execute_write(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationWriteMarkerStage,
            |inner| inner.write_marker_stage(marker),
            |inner| {
                inner.write_fixed_stage_prefix_for_repository_tasks(
                    StoreMigrationFixedStage::Marker,
                    marker.encoded(),
                    marker.encoded().len() / 2,
                )
            },
        )
    }

    fn synchronize_marker_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeMarkerStage,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_marker_stage,
        )
    }

    fn link_marker(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationLinkMarker,
            DuringTiming::After,
            |inner| inner.link_marker(marker),
        )
    }

    fn synchronize_root_after_marker(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterMarker,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_marker,
        )
    }

    fn remove_marker_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationRemoveMarkerStage,
            DuringTiming::After,
            FilesystemStoreMigrationAuthority::remove_marker_stage,
        )
    }

    fn synchronize_root_after_marker_cleanup(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterMarkerCleanup,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_marker_cleanup,
        )
    }

    fn write_receipt_stage(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        execute_write(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationWriteReceiptStage,
            |inner| inner.write_receipt_stage(receipt),
            |inner| {
                inner.write_fixed_stage_prefix_for_repository_tasks(
                    StoreMigrationFixedStage::Receipt,
                    receipt.encoded(),
                    receipt.encoded().len() / 2,
                )
            },
        )
    }

    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeReceiptStage,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_receipt_stage,
        )
    }

    fn link_receipt(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationLinkReceipt,
            DuringTiming::After,
            |inner| inner.link_receipt(receipt),
        )
    }

    fn synchronize_root_after_receipt(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterReceipt,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_receipt,
        )
    }

    fn remove_receipt_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationRemoveReceiptStage,
            DuringTiming::After,
            FilesystemStoreMigrationAuthority::remove_receipt_stage,
        )
    }

    fn synchronize_root_after_receipt_cleanup(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::MigrationSynchronizeRootAfterReceiptCleanup,
            DuringTiming::Before,
            FilesystemStoreMigrationAuthority::synchronize_root_after_receipt_cleanup,
        )
    }
}

fn execute<T>(
    inner: &mut FilesystemStoreMigrationAuthority,
    control: &mut CrashControl,
    point: DurabilityCrashPoint,
    during: DuringTiming,
    operation: impl FnOnce(&mut FilesystemStoreMigrationAuthority) -> io::Result<T>,
) -> io::Result<T> {
    control.before(point, during)?;
    let result = operation(inner)?;
    control.after(point, during)?;
    Ok(result)
}

fn execute_write(
    inner: &mut FilesystemStoreMigrationAuthority,
    control: &mut CrashControl,
    point: DurabilityCrashPoint,
    complete: impl FnOnce(&mut FilesystemStoreMigrationAuthority) -> io::Result<()>,
    interrupted: impl FnOnce(&mut FilesystemStoreMigrationAuthority) -> io::Result<()>,
) -> io::Result<()> {
    match control.position(point) {
        None => complete(inner),
        Some(DurabilityCrashPosition::Before) => control.await_process_death(),
        Some(DurabilityCrashPosition::During) => {
            interrupted(inner)?;
            control.await_process_death()
        }
        Some(DurabilityCrashPosition::After) => {
            complete(inner)?;
            control.await_process_death()
        }
    }
}
