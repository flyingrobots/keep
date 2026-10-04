//! This module owns progress-preserving dispatch of GC storage capabilities.

use super::{FilesystemGcAuthority, GcExecutionStorage};
use crate::adapters::retention::{
    RetentionStorageBoundary as Boundary, RetentionStorageError as StorageError,
};
use std::io;

impl FilesystemGcAuthority {
    #[cfg_attr(
        not(test),
        expect(
            clippy::unused_self,
            clippy::missing_const_for_fn,
            clippy::unnecessary_wraps,
            reason = "the production checkpoint is inert; test builds inject deterministic syscall failures"
        )
    )]
    pub(super) fn storage_checkpoint(&self, boundary: Boundary) -> io::Result<()> {
        #[cfg(test)]
        if self.storage_failure == Some(boundary) {
            return Err(io::Error::from_raw_os_error(5));
        }
        let _ = boundary;
        Ok(())
    }
}

impl GcExecutionStorage for FilesystemGcAuthority {
    fn write_intent_stage(&mut self) -> io::Result<()> {
        self.store_write_intent_stage()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.store_synchronize_intent_stage()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn link_intent(&mut self) -> io::Result<()> {
        self.store_link_intent()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_gc_after_intent(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent()
            .map_err(|source| at(source, Boundary::GcIntentSynchronization))
    }
    fn remove_intent_stage(&mut self) -> io::Result<()> {
        self.store_remove_intent_stage()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_gc_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent_cleanup()
            .map_err(|source| at(source, Boundary::GcIntentCleanupSynchronization))
    }
    fn unlink_candidate(&mut self, index: usize) -> io::Result<()> {
        self.store_unlink_candidate(index)
            .map_err(|source| at(source, Boundary::CandidateVerification))
    }
    fn synchronize_segment_pool(&mut self, index: usize) -> io::Result<()> {
        self.store_synchronize_segment_pool(index)
            .map_err(|source| at(source, Boundary::PoolSynchronization))
    }
    fn write_receipt_stage(&mut self) -> io::Result<()> {
        self.store_write_receipt_stage()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.store_synchronize_receipt_stage()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn replace_receipt(&mut self) -> io::Result<()> {
        self.store_replace_receipt()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_gc_after_receipt(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_receipt()
            .map_err(|source| at(source, Boundary::GcReceiptSynchronization))
    }
    fn remove_intent(&mut self) -> io::Result<()> {
        self.store_remove_intent()
            .map_err(|source| at(source, Boundary::Preparation))
    }
    fn synchronize_gc_after_intent_removal(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent_removal()
            .map_err(|source| at(source, Boundary::GcIntentRemovalSynchronization))
    }
}

/// Adds a boundary only when a deeper capability has not already reported one.
pub(super) fn at(source: io::Error, boundary: Boundary) -> io::Error {
    if source
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<StorageError>())
        .and_then(StorageError::progress)
        .is_some()
    {
        source
    } else {
        StorageError::from(source).at(boundary).into()
    }
}
