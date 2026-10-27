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
    /// Version-two retention publication, `KEEP-CRASH-036` through `052`.
    Retention,
    /// Direct version-two initialization and its recovery, `KEEP-CRASH-081`
    /// through `091`.
    VersionTwoInitialization,
    /// Version-two catalog publication and its stage recovery, `KEEP-CRASH-074`
    /// through `077`.
    VersionTwoCatalog,
}

impl DurabilityCrashSequence {
    /// Every sequence in stable protocol order.
    pub const ALL: [Self; 8] = [
        Self::Segment,
        Self::Catalog,
        Self::Head,
        Self::RecoveryDiscard,
        Self::Initialization,
        Self::Retention,
        Self::VersionTwoCatalog,
        Self::VersionTwoInitialization,
    ];

    /// Returns the stable identifier used by the crash-matrix command line.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Segment => "segment",
            Self::Catalog => "catalog",
            Self::Head => "head",
            Self::RecoveryDiscard => "recovery-discard",
            Self::Initialization => "initialization",
            Self::Retention => "retention",
            Self::VersionTwoCatalog => "version-two-catalog",
            Self::VersionTwoInitialization => "version-two-initialization",
        }
    }

    /// Parses one exact sequence identifier.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|sequence| sequence.identifier() == identifier)
    }
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
    /// Retention root stage write.
    WriteRootStage,
    /// Retention root stage synchronization.
    SynchronizeRootStage,
    /// New namespace-directory creation or exact admission.
    AdmitRootNamespace,
    /// Namespace-pool synchronization after creation.
    SynchronizeRootsAfterNamespace,
    /// Immutable root link.
    LinkRoot,
    /// Root namespace-directory synchronization.
    SynchronizeRootNamespace,
    /// Retention manifest stage write.
    WriteManifestStage,
    /// Retention manifest stage synchronization.
    SynchronizeManifestStage,
    /// Immutable manifest link.
    LinkManifest,
    /// Manifest pool synchronization.
    SynchronizeManifestPool,
    /// Retention-head stage write.
    WriteHeadStage,
    /// Retention-head stage synchronization.
    SynchronizeHeadStage,
    /// Retention-head atomic replacement.
    ReplaceRetentionHead,
    /// Committed retention namespace synchronization.
    SynchronizeRetentionNamespace,
    /// Retained root-stage removal.
    RemoveRootStage,
    /// Retained manifest-stage removal.
    RemoveManifestStage,
    /// Retention cleanup synchronization.
    SynchronizeRetentionCleanup,
    /// Link the version-two writer's sealed segment into the immutable pool.
    VersionTwoLinkSegment,
    /// Exclusively create the version-two writer's `staging/current.cat`.
    VersionTwoCreateCatalogStage,
    /// Synchronize the version-two writer's complete `head.next`.
    VersionTwoSynchronizeHead,
    /// Replace `HEAD` after the version-two retained-closure revalidation.
    VersionTwoReplaceHead,
    /// Create or open `writer.lock`, lock it, and synchronize it and the root.
    InitializeOpenAndLockWriterFile,
    /// Exclusively create `FORMAT.next` and write the canonical marker; the
    /// `during` case leaves a strict prefix.
    InitializeWriteMarkerStage,
    /// Synchronize `FORMAT.next`.
    InitializeSynchronizeMarkerStage,
    /// Synchronize the root after the marker stage.
    InitializeSynchronizeRootAfterMarkerStage,
    /// Create or admit `reader.lock` and synchronize the root.
    InitializeAdmitReaderFence,
    /// Create or admit one protocol directory and synchronize its parent; the
    /// occurrence names the directory reached.
    InitializeAdmitDirectory,
    /// Link `FORMAT.next` to `FORMAT` without replacement.
    InitializeLinkMarker,
    /// Synchronize the root after the link: the commit point.
    InitializeSynchronizeRootAfterMarker,
    /// Remove `FORMAT.next` after the commit.
    InitializeRemoveMarkerStage,
    /// Synchronize the root after stage removal.
    InitializeSynchronizeRootAfterCleanup,
    /// Discard an incomplete `FORMAT.next` while recovering an interrupted
    /// initialization.
    InitializeDiscardIncompleteMarkerStage,
}

impl DurabilityCrashPoint {
    /// Every crash boundary in stable protocol order.
    pub const ALL: [Self; 67] = [
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
        Self::WriteRootStage,
        Self::SynchronizeRootStage,
        Self::AdmitRootNamespace,
        Self::SynchronizeRootsAfterNamespace,
        Self::LinkRoot,
        Self::SynchronizeRootNamespace,
        Self::WriteManifestStage,
        Self::SynchronizeManifestStage,
        Self::LinkManifest,
        Self::SynchronizeManifestPool,
        Self::WriteHeadStage,
        Self::SynchronizeHeadStage,
        Self::ReplaceRetentionHead,
        Self::SynchronizeRetentionNamespace,
        Self::RemoveRootStage,
        Self::RemoveManifestStage,
        Self::SynchronizeRetentionCleanup,
        Self::VersionTwoLinkSegment,
        Self::VersionTwoCreateCatalogStage,
        Self::VersionTwoSynchronizeHead,
        Self::VersionTwoReplaceHead,
        Self::InitializeOpenAndLockWriterFile,
        Self::InitializeWriteMarkerStage,
        Self::InitializeSynchronizeMarkerStage,
        Self::InitializeSynchronizeRootAfterMarkerStage,
        Self::InitializeAdmitReaderFence,
        Self::InitializeAdmitDirectory,
        Self::InitializeLinkMarker,
        Self::InitializeSynchronizeRootAfterMarker,
        Self::InitializeRemoveMarkerStage,
        Self::InitializeSynchronizeRootAfterCleanup,
        Self::InitializeDiscardIncompleteMarkerStage,
    ];

    /// The number of protocol directories initialization creates; each is one
    /// `during` occurrence of [`Self::InitializeAdmitDirectory`].
    pub const INITIALIZATION_DIRECTORIES: u32 = 9;

    /// Parses one exact stable crash identifier.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|point| point.identifier() == identifier)
    }

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
            Self::WriteRootStage
            | Self::SynchronizeRootStage
            | Self::AdmitRootNamespace
            | Self::SynchronizeRootsAfterNamespace
            | Self::LinkRoot
            | Self::SynchronizeRootNamespace
            | Self::WriteManifestStage
            | Self::SynchronizeManifestStage
            | Self::LinkManifest
            | Self::SynchronizeManifestPool
            | Self::WriteHeadStage
            | Self::SynchronizeHeadStage
            | Self::ReplaceRetentionHead
            | Self::SynchronizeRetentionNamespace
            | Self::RemoveRootStage
            | Self::RemoveManifestStage
            | Self::SynchronizeRetentionCleanup => DurabilityCrashSequence::Retention,
            Self::InitializeOpenAndLockWriterFile
            | Self::InitializeWriteMarkerStage
            | Self::InitializeSynchronizeMarkerStage
            | Self::InitializeSynchronizeRootAfterMarkerStage
            | Self::InitializeAdmitReaderFence
            | Self::InitializeAdmitDirectory
            | Self::InitializeLinkMarker
            | Self::InitializeSynchronizeRootAfterMarker
            | Self::InitializeRemoveMarkerStage
            | Self::InitializeSynchronizeRootAfterCleanup
            | Self::InitializeDiscardIncompleteMarkerStage => {
                DurabilityCrashSequence::VersionTwoInitialization
            }
            Self::VersionTwoLinkSegment
            | Self::VersionTwoCreateCatalogStage
            | Self::VersionTwoSynchronizeHead
            | Self::VersionTwoReplaceHead => DurabilityCrashSequence::VersionTwoCatalog,
        }
    }

    /// Reports whether tests may select a repeated occurrence.
    #[must_use]
    pub const fn occurrence_counted(self) -> bool {
        matches!(
            self,
            Self::AppendSegmentRecord | Self::InitializeAdmitDirectory
        )
    }

    /// Returns how many distinct `during` occurrences the canonical matrix
    /// runs for this boundary: one directory per occurrence for initialization
    /// directory admission, otherwise exactly one.
    #[must_use]
    pub const fn during_occurrences(self) -> u32 {
        match self {
            Self::InitializeAdmitDirectory => Self::INITIALIZATION_DIRECTORIES,
            _ => 1,
        }
    }
}
