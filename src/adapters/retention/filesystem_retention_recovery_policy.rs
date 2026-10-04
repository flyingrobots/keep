//! This module owns the default catalog resource policy for retention authority.

use std::io;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::{FilesystemRetentionRecoveryError, RetentionRecoveryReceipt};
use crate::adapters::segment_header::MAXIMUM_SEGMENT_LENGTH;
use crate::adapters::{CatalogRestartByteLimit, CatalogRestartPolicy, SegmentReadPolicy};

impl FilesystemRetentionPublicationAuthority {
    /// Observes, plans, and executes fixed-stage recovery under writer authority.
    ///
    /// Complete staged roots materialize the current catalog and all selected
    /// segments before closure verification. Aggregate retained segment bytes
    /// are bounded to one protocol-maximum segment (1 GiB), independently of
    /// the root's own closure limits; catalog and decoded indexes also retain
    /// their protocol bounds. Larger catalog selections refuse without mutation.
    /// Use [`Self::recover_with_catalog_policy`] for explicit caller-selected
    /// loading bounds. Clean and incomplete-root recovery does not load segments.
    /// The synchronous call may allocate and block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns the exact observation, planning, resource, closure, or execution
    /// refusal; catalog and closure errors remain inspectable as error sources.
    pub fn recover(
        &mut self,
    ) -> Result<RetentionRecoveryReceipt, FilesystemRetentionRecoveryError> {
        let policy = default_catalog_policy()
            .map_err(|source| FilesystemRetentionRecoveryError::Observe { source })?;
        self.recover_with_catalog_policy(policy)
    }
}

pub(super) fn default_catalog_policy() -> io::Result<CatalogRestartPolicy> {
    let bound = CatalogRestartByteLimit::new(MAXIMUM_SEGMENT_LENGTH)
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidInput, source))?;
    Ok(CatalogRestartPolicy::new(SegmentReadPolicy::MAXIMUM, bound))
}
