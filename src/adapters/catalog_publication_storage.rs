//! Blocking durability port for one writer-locked catalog publication.

use std::io;

use super::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationReadiness, CatalogSnapshot, ChecksummedCatalog,
};

/// Blocking filesystem capabilities for the catalog publication protocol.
///
/// An implementation must retain exclusive writer authority and one pinned
/// store root for the complete call. Methods correspond to exact protocol
/// transitions and must not combine later transitions or report success before
/// the named durability or verification obligation is satisfied.
pub trait CatalogPublicationStorage {
    /// Reopens and verifies the expected predecessor or complete candidate.
    ///
    /// Returns [`CatalogPublicationReadiness::Ready`] only when `expected` is
    /// current. Returns [`CatalogPublicationReadiness::AlreadyPublished`] only
    /// when the exact generation and digest in `candidate` are current.
    ///
    /// # Errors
    ///
    /// Returns the exact current-state verification failure.
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<CatalogPublicationReadiness>;

    /// Links the exact sealed stage without replacing an immutable pool entry.
    ///
    /// # Errors
    ///
    /// Returns the exact link failure.
    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()>;

    /// Reopens and completely verifies the resolved immutable segment.
    ///
    /// # Errors
    ///
    /// Returns the exact reopen or verification failure.
    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()>;

    /// Synchronizes the segment-pool directory.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_segments(&mut self) -> io::Result<()>;

    /// Removes only the fixed segment staging name.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_segment_stage(&mut self) -> io::Result<()>;

    /// Synchronizes staging after segment removal.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_staging_after_segment(&mut self) -> io::Result<()>;

    /// Exclusively creates the fixed empty catalog stage.
    ///
    /// # Errors
    ///
    /// Returns the exact exclusive-creation failure.
    fn create_catalog_stage(&mut self) -> io::Result<()>;

    /// Writes the complete canonical catalog.
    ///
    /// # Errors
    ///
    /// Returns the exact write failure.
    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()>;

    /// Flushes the complete catalog stage.
    ///
    /// # Errors
    ///
    /// Returns the exact flush failure.
    fn flush_catalog(&mut self) -> io::Result<()>;

    /// Synchronizes the complete catalog stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_catalog(&mut self) -> io::Result<()>;

    /// Links the catalog stage without replacing an immutable pool entry.
    ///
    /// # Errors
    ///
    /// Returns the exact link failure.
    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()>;

    /// Reopens and completely verifies the resolved immutable catalog.
    ///
    /// # Errors
    ///
    /// Returns the exact reopen or verification failure.
    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()>;

    /// Synchronizes the catalog-pool directory.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_catalogs(&mut self) -> io::Result<()>;

    /// Removes only the fixed catalog staging name.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_catalog_stage(&mut self) -> io::Result<()>;

    /// Synchronizes staging after catalog removal.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()>;

    /// Exclusively creates the fixed empty `head.next`.
    ///
    /// # Errors
    ///
    /// Returns the exact exclusive-creation failure.
    fn create_head_stage(&mut self) -> io::Result<()>;

    /// Writes the complete canonical next head.
    ///
    /// # Errors
    ///
    /// Returns the exact write failure.
    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()>;

    /// Flushes the complete next head.
    ///
    /// # Errors
    ///
    /// Returns the exact flush failure.
    fn flush_head(&mut self) -> io::Result<()>;

    /// Synchronizes the complete next head.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_head(&mut self) -> io::Result<()>;

    /// Reopens and verifies the exact complete transitive next-head view.
    ///
    /// # Errors
    ///
    /// Returns the exact reopen or verification failure.
    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()>;

    /// Atomically replaces `HEAD` with the verified `head.next`.
    ///
    /// # Errors
    ///
    /// Returns the exact atomic-replacement failure.
    fn replace_head(&mut self) -> io::Result<()>;

    /// Synchronizes the store root after head replacement.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root(&mut self) -> io::Result<()>;
}
