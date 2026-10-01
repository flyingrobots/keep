//! This module owns crash injection around production GC retirement.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use keep::{FilesystemGcAuthority, GcExecutionStorage};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

/// Bytes an interrupted stage write leaves behind: inside both records'
/// fixed framing, so restart classifies the stage as truncated.
const STAGE_INTERRUPTION: usize = 100;

pub(super) struct CrashGcStorage<'control> {
    inner: FilesystemGcAuthority,
    control: &'control mut CrashControl,
    gc: PathBuf,
}

impl<'control> CrashGcStorage<'control> {
    pub(super) fn new(
        inner: FilesystemGcAuthority,
        control: &'control mut CrashControl,
        store_root: &Path,
    ) -> Self {
        Self {
            inner,
            control,
            gc: store_root.join("gc"),
        }
    }

    fn execute(
        &mut self,
        point: DurabilityCrashPoint,
        during: DuringTiming,
        operation: impl FnOnce(&mut FilesystemGcAuthority) -> io::Result<()>,
    ) -> io::Result<()> {
        self.control.before(point, during)?;
        operation(&mut self.inner)?;
        self.control.after(point, during)
    }

    /// A stage write dies before, mid-record, or after the complete write;
    /// mid-record leaves the record's first bytes in the stage.
    fn execute_write(
        &mut self,
        point: DurabilityCrashPoint,
        stage: &str,
        prefix: &[u8],
        complete: impl FnOnce(&mut FilesystemGcAuthority) -> io::Result<()>,
    ) -> io::Result<()> {
        match self.control.position(point) {
            None => complete(&mut self.inner),
            Some(DurabilityCrashPosition::Before) => self.control.await_process_death(),
            Some(DurabilityCrashPosition::During) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.gc.join(stage))?;
                file.write_all(prefix)?;
                self.control.await_process_death()
            }
            Some(DurabilityCrashPosition::After) => {
                complete(&mut self.inner)?;
                self.control.await_process_death()
            }
        }
    }

    fn intent_prefix(&self) -> io::Result<Vec<u8>> {
        let intent = self
            .inner
            .bound_intent()
            .ok_or_else(|| io::Error::other("no GC intent is bound"))?;
        intent
            .encoded()
            .get(..STAGE_INTERRUPTION)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| io::Error::other("GC intent shorter than the interruption prefix"))
    }
}

/// The receipt's magic followed by zeros: a truncated receipt stage.
fn receipt_prefix() -> Vec<u8> {
    let mut prefix = b"KEEP:GC:RECEIPT2".to_vec();
    prefix.resize(STAGE_INTERRUPTION, 0);
    prefix
}

impl GcExecutionStorage for CrashGcStorage<'_> {
    fn write_intent_stage(&mut self) -> io::Result<()> {
        let prefix = self.intent_prefix()?;
        self.execute_write(
            DurabilityCrashPoint::GcWriteIntentStage,
            "intent.next",
            &prefix,
            FilesystemGcAuthority::write_intent_stage,
        )
    }

    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeIntentStage,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_intent_stage,
        )
    }

    fn link_intent(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcLinkIntent,
            DuringTiming::After,
            FilesystemGcAuthority::link_intent,
        )
    }

    fn synchronize_gc_after_intent(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeGcAfterIntent,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_gc_after_intent,
        )
    }

    fn remove_intent_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcRemoveIntentStage,
            DuringTiming::After,
            FilesystemGcAuthority::remove_intent_stage,
        )
    }

    fn synchronize_gc_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeGcAfterIntentCleanup,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_gc_after_intent_cleanup,
        )
    }

    fn unlink_candidate(&mut self, index: usize) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcUnlinkCandidate,
            DuringTiming::After,
            |inner| inner.unlink_candidate(index),
        )
    }

    fn synchronize_segment_pool(&mut self, index: usize) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeSegmentPool,
            DuringTiming::Before,
            |inner| inner.synchronize_segment_pool(index),
        )
    }

    fn write_receipt_stage(&mut self) -> io::Result<()> {
        self.execute_write(
            DurabilityCrashPoint::GcWriteReceiptStage,
            "receipt.next",
            &receipt_prefix(),
            FilesystemGcAuthority::write_receipt_stage,
        )
    }

    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeReceiptStage,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_receipt_stage,
        )
    }

    fn replace_receipt(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcReplaceReceipt,
            DuringTiming::After,
            FilesystemGcAuthority::replace_receipt,
        )
    }

    fn synchronize_gc_after_receipt(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeGcAfterReceipt,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_gc_after_receipt,
        )
    }

    fn remove_intent(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcRemoveIntent,
            DuringTiming::After,
            FilesystemGcAuthority::remove_intent,
        )
    }

    fn synchronize_gc_after_intent_removal(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::GcSynchronizeGcAfterIntentRemoval,
            DuringTiming::Before,
            FilesystemGcAuthority::synchronize_gc_after_intent_removal,
        )
    }
}
