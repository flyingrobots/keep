//! Stable identifiers for deterministic segment-store crash injection.

/// Durable protocol sequence containing a crash boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityCrashSequence {
    /// Segment staging, sealing, and immutable-pool publication.
    Segment,
    /// Catalog staging and immutable-pool publication.
    Catalog,
    /// Publication-head staging and replacement.
    Head,
    /// Explicit discard of fingerprint-bound recovery evidence.
    RecoveryDiscard,
    /// Writer-locked store initialization.
    Initialization,
}

/// One stable process-death boundary in the durable segment-store protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityCrashPoint {
    /// Exclusively create `staging/current.seg`.
    CreateSegmentStage,
    /// Write the complete segment header.
    WriteSegmentHeader,
    /// Append one complete segment record and checksum.
    AppendSegmentRecord,
    /// Flush the reusable record prefix.
    FlushSegmentRecordPrefix,
    /// Synchronize the reusable record prefix.
    SynchronizeSegmentRecordPrefix,
    /// Append the complete segment seal.
    AppendSegmentSeal,
    /// Flush the sealed segment bytes.
    FlushSealedSegment,
    /// Synchronize the sealed segment stage.
    SynchronizeSealedSegment,
    /// Verify and link the segment into the immutable pool.
    LinkSegment,
    /// Synchronize the segment-pool directory.
    SynchronizeSegmentPool,
    /// Remove `staging/current.seg`.
    RemoveSegmentStage,
    /// Synchronize staging after segment-stage removal.
    SynchronizeStagingAfterSegment,
    /// Exclusively create `staging/current.cat`.
    CreateCatalogStage,
    /// Write the complete canonical catalog.
    WriteCatalog,
    /// Flush the complete catalog.
    FlushCatalog,
    /// Synchronize the catalog stage.
    SynchronizeCatalog,
    /// Verify and link the catalog into the immutable pool.
    LinkCatalog,
    /// Synchronize the catalog-pool directory.
    SynchronizeCatalogPool,
    /// Remove `staging/current.cat`.
    RemoveCatalogStage,
    /// Synchronize staging after catalog-stage removal.
    SynchronizeStagingAfterCatalog,
    /// Exclusively create `head.next`.
    CreateHeadStage,
    /// Write the complete next publication head.
    WriteHead,
    /// Flush the complete next publication head.
    FlushHead,
    /// Synchronize `head.next`.
    SynchronizeHead,
    /// Verify `head.next` and replace `HEAD`.
    ReplaceHead,
    /// Synchronize the store root after head replacement.
    SynchronizeRootAfterHead,
    /// Remove a fingerprint-bound segment or catalog recovery stage.
    RemoveRecoveryStage,
    /// Synchronize staging after recovery-stage removal.
    SynchronizeStagingAfterRecovery,
    /// Remove fingerprint-bound `head.next` recovery evidence.
    RemoveRecoveryHead,
    /// Synchronize the store root after recovery-head removal.
    SynchronizeRootAfterRecovery,
    /// Create or reopen `writer.lock`, then acquire writer authority.
    OpenAndLockWriterFile,
    /// Create or verify the staging directory.
    CreateStagingDirectory,
    /// Create or verify the segment-pool directory.
    CreateSegmentPoolDirectory,
    /// Create or verify the catalog-pool directory.
    CreateCatalogPoolDirectory,
    /// Synchronize the store root after initialization.
    SynchronizeRootAfterInitialization,
}

impl DurabilityCrashPoint {
    /// Every crash boundary in stable protocol order.
    pub const ALL: [Self; 35] = [
        Self::CreateSegmentStage,
        Self::WriteSegmentHeader,
        Self::AppendSegmentRecord,
        Self::FlushSegmentRecordPrefix,
        Self::SynchronizeSegmentRecordPrefix,
        Self::AppendSegmentSeal,
        Self::FlushSealedSegment,
        Self::SynchronizeSealedSegment,
        Self::LinkSegment,
        Self::SynchronizeSegmentPool,
        Self::RemoveSegmentStage,
        Self::SynchronizeStagingAfterSegment,
        Self::CreateCatalogStage,
        Self::WriteCatalog,
        Self::FlushCatalog,
        Self::SynchronizeCatalog,
        Self::LinkCatalog,
        Self::SynchronizeCatalogPool,
        Self::RemoveCatalogStage,
        Self::SynchronizeStagingAfterCatalog,
        Self::CreateHeadStage,
        Self::WriteHead,
        Self::FlushHead,
        Self::SynchronizeHead,
        Self::ReplaceHead,
        Self::SynchronizeRootAfterHead,
        Self::RemoveRecoveryStage,
        Self::SynchronizeStagingAfterRecovery,
        Self::RemoveRecoveryHead,
        Self::SynchronizeRootAfterRecovery,
        Self::OpenAndLockWriterFile,
        Self::CreateStagingDirectory,
        Self::CreateSegmentPoolDirectory,
        Self::CreateCatalogPoolDirectory,
        Self::SynchronizeRootAfterInitialization,
    ];

    /// Returns the durable protocol sequence containing this boundary.
    #[must_use]
    pub const fn sequence(self) -> DurabilityCrashSequence {
        match self {
            Self::CreateSegmentStage
            | Self::WriteSegmentHeader
            | Self::AppendSegmentRecord
            | Self::FlushSegmentRecordPrefix
            | Self::SynchronizeSegmentRecordPrefix
            | Self::AppendSegmentSeal
            | Self::FlushSealedSegment
            | Self::SynchronizeSealedSegment
            | Self::LinkSegment
            | Self::SynchronizeSegmentPool
            | Self::RemoveSegmentStage
            | Self::SynchronizeStagingAfterSegment => DurabilityCrashSequence::Segment,
            Self::CreateCatalogStage
            | Self::WriteCatalog
            | Self::FlushCatalog
            | Self::SynchronizeCatalog
            | Self::LinkCatalog
            | Self::SynchronizeCatalogPool
            | Self::RemoveCatalogStage
            | Self::SynchronizeStagingAfterCatalog => DurabilityCrashSequence::Catalog,
            Self::CreateHeadStage
            | Self::WriteHead
            | Self::FlushHead
            | Self::SynchronizeHead
            | Self::ReplaceHead
            | Self::SynchronizeRootAfterHead => DurabilityCrashSequence::Head,
            Self::RemoveRecoveryStage
            | Self::SynchronizeStagingAfterRecovery
            | Self::RemoveRecoveryHead
            | Self::SynchronizeRootAfterRecovery => DurabilityCrashSequence::RecoveryDiscard,
            Self::OpenAndLockWriterFile
            | Self::CreateStagingDirectory
            | Self::CreateSegmentPoolDirectory
            | Self::CreateCatalogPoolDirectory
            | Self::SynchronizeRootAfterInitialization => DurabilityCrashSequence::Initialization,
        }
    }

    /// Reports whether tests may select a repeated occurrence.
    #[must_use]
    pub const fn occurrence_counted(self) -> bool {
        matches!(self, Self::AppendSegmentRecord)
    }
}
