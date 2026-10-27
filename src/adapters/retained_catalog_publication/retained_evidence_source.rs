//! This module owns the crate-internal capability to read retention evidence.
//!
//! The retained-closure gate needs exact retention bytes on two routes: forward
//! catalog publication, through [`RetainedCatalogPublicationStorage`], and
//! recovery finalization of a staged catalog head, which owns no catalog
//! stages. Both routes supply evidence through this one capability so the same
//! admission code decodes, binds, and verifies it.

use std::io;

use super::RetainedCatalogPublicationStorage;
use crate::{RetentionHead, RetentionManifestEntry};

/// Exact retention record bytes read under writer authority.
///
/// The contract of each method is the contract of the identically named
/// [`RetainedCatalogPublicationStorage`] method: exact bytes, no-follow and
/// bounded reads, and `None` for the head only when nothing contradicts that
/// absence.
pub(in crate::adapters) trait RetainedEvidenceSource {
    /// Reads the exact current retention head bytes, if one is published.
    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>>;

    /// Reads the exact immutable manifest that `head` selects.
    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>>;

    /// Reads the exact immutable root that one manifest entry selects.
    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>>;
}

/// The evidence reads of one publication storage, without its catalog stages.
pub(super) struct PublicationEvidence<'storage, S>(pub(super) &'storage mut S);

impl<S: RetainedCatalogPublicationStorage> RetainedEvidenceSource for PublicationEvidence<'_, S> {
    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.0.read_retention_head()
    }

    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>> {
        self.0.read_retention_manifest(head)
    }

    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>> {
        self.0.read_retained_root(entry)
    }
}
