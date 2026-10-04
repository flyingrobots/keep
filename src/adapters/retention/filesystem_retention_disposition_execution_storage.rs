//! This module owns progress-preserving dispatch of disposition capabilities.

use super::{
    FilesystemRetentionPublicationAuthority, RecoveryDispositionStorage,
    RetentionStorageBoundary as Boundary,
};
use std::io;

impl FilesystemRetentionPublicationAuthority {
    #[cfg_attr(
        not(test),
        expect(
            clippy::unused_self,
            clippy::missing_const_for_fn,
            clippy::unnecessary_wraps,
            reason = "production checkpoints are inert; tests inject deterministic I/O failures"
        )
    )]
    pub(super) fn disposition_checkpoint(&self, boundary: Boundary) -> io::Result<()> {
        #[cfg(test)]
        if self.disposition_failure == Some(boundary) {
            return Err(io::Error::from_raw_os_error(5));
        }
        let _ = boundary;
        Ok(())
    }
}

impl RecoveryDispositionStorage for FilesystemRetentionPublicationAuthority {
    fn write_disposition_stage(&mut self) -> io::Result<()> {
        self.store_write_disposition_stage()
    }
    fn synchronize_disposition_stage(&mut self) -> io::Result<()> {
        self.store_synchronize_disposition_stage()
    }
    fn link_disposition_receipt(&mut self) -> io::Result<()> {
        self.store_link_disposition_receipt()
    }
    fn synchronize_dispositions(&mut self) -> io::Result<()> {
        self.store_synchronize_dispositions()
    }
    fn remove_disposition_stage(&mut self) -> io::Result<()> {
        self.store_remove_disposition_stage()
    }
    fn synchronize_recovery(&mut self) -> io::Result<()> {
        self.store_synchronize_recovery()
    }
    fn remove_retained_stage(&mut self) -> io::Result<()> {
        self.store_remove_retained_stage()
    }
    fn synchronize_retention_after_disposition(&mut self) -> io::Result<()> {
        self.store_synchronize_retention_after_disposition()
    }
    fn remove_pool_entry(&mut self) -> io::Result<()> {
        self.store_remove_pool_entry()
    }
    fn synchronize_pool(&mut self) -> io::Result<()> {
        self.store_synchronize_pool()
    }
}
