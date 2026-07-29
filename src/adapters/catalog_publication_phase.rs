//! Exact catalog-generation publication durability phases.

use std::fmt;

/// Filesystem transition attempted by catalog-generation publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPublicationPhase {
    /// Reopen and verify the expected current head and catalog.
    VerifyCurrent,
    /// Link the synchronized staged segment into the immutable pool.
    LinkSegment,
    /// Reopen and verify the resolved immutable segment.
    VerifySegmentPool,
    /// Synchronize the segment-pool directory.
    SynchronizeSegments,
    /// Remove the fixed segment staging name.
    RemoveSegmentStage,
    /// Synchronize staging after segment removal.
    SynchronizeStagingAfterSegment,
    /// Exclusively create the fixed catalog staging name.
    CreateCatalogStage,
    /// Write the complete canonical catalog.
    WriteCatalog,
    /// Flush the complete catalog.
    FlushCatalog,
    /// Synchronize the catalog staging file.
    SynchronizeCatalog,
    /// Link the verified catalog into the immutable pool.
    LinkCatalog,
    /// Reopen and verify the resolved immutable catalog.
    VerifyCatalogPool,
    /// Synchronize the catalog-pool directory.
    SynchronizeCatalogs,
    /// Remove the fixed catalog staging name.
    RemoveCatalogStage,
    /// Synchronize staging after catalog removal.
    SynchronizeStagingAfterCatalog,
    /// Exclusively create `head.next`.
    CreateHeadStage,
    /// Write the complete canonical next head.
    WriteHead,
    /// Flush the complete next head.
    FlushHead,
    /// Synchronize `head.next`.
    SynchronizeHead,
    /// Reopen and verify the complete transitive head view.
    VerifyHeadView,
    /// Atomically replace `HEAD` with `head.next`.
    ReplaceHead,
    /// Synchronize the store root.
    SynchronizeRoot,
}

impl fmt::Display for CatalogPublicationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::VerifyCurrent => "current-head verification",
            Self::LinkSegment => "segment link",
            Self::VerifySegmentPool => "segment-pool verification",
            Self::SynchronizeSegments => "segment-directory synchronization",
            Self::RemoveSegmentStage => "segment-stage removal",
            Self::SynchronizeStagingAfterSegment => "post-segment staging synchronization",
            Self::CreateCatalogStage => "catalog-stage creation",
            Self::WriteCatalog => "catalog write",
            Self::FlushCatalog => "catalog flush",
            Self::SynchronizeCatalog => "catalog synchronization",
            Self::LinkCatalog => "catalog link",
            Self::VerifyCatalogPool => "catalog-pool verification",
            Self::SynchronizeCatalogs => "catalog-directory synchronization",
            Self::RemoveCatalogStage => "catalog-stage removal",
            Self::SynchronizeStagingAfterCatalog => "post-catalog staging synchronization",
            Self::CreateHeadStage => "next-head creation",
            Self::WriteHead => "next-head write",
            Self::FlushHead => "next-head flush",
            Self::SynchronizeHead => "next-head synchronization",
            Self::VerifyHeadView => "next-head view verification",
            Self::ReplaceHead => "head replacement",
            Self::SynchronizeRoot => "root synchronization",
        })
    }
}
