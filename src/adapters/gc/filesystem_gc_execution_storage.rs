//! This module owns progress-preserving dispatch of GC storage capabilities.

use std::io;
use super::{FilesystemGcAuthority, GcExecutionStorage};
use crate::adapters::retention::RetentionStorageBoundary as Boundary;

impl FilesystemGcAuthority {
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
    }
    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.store_synchronize_intent_stage()
    }
    fn link_intent(&mut self) -> io::Result<()> {
        self.store_link_intent()
    }
    fn synchronize_gc_after_intent(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent()
    }
    fn remove_intent_stage(&mut self) -> io::Result<()> {
        self.store_remove_intent_stage()
    }
    fn synchronize_gc_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent_cleanup()
    }
    fn unlink_candidate(&mut self, index: usize) -> io::Result<()> {
        self.store_unlink_candidate(index)
    }
    fn synchronize_segment_pool(&mut self, index: usize) -> io::Result<()> {
        self.store_synchronize_segment_pool(index)
    }
    fn write_receipt_stage(&mut self) -> io::Result<()> {
        self.store_write_receipt_stage()
    }
    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.store_synchronize_receipt_stage()
    }
    fn replace_receipt(&mut self) -> io::Result<()> {
        self.store_replace_receipt()
    }
    fn synchronize_gc_after_receipt(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_receipt()
    }
    fn remove_intent(&mut self) -> io::Result<()> {
        self.store_remove_intent()
    }
    fn synchronize_gc_after_intent_removal(&mut self) -> io::Result<()> {
        self.store_synchronize_gc_after_intent_removal()
    }
}
