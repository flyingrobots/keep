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
    /// One-way version-1 to version-2 store migration.
    Migration,
}

impl DurabilityCrashSequence {
    /// Every sequence in stable protocol order.
    pub const ALL: [Self; 6] = [
        Self::Segment,
        Self::Catalog,
        Self::Head,
        Self::RecoveryDiscard,
        Self::Initialization,
        Self::Migration,
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
            Self::Migration => "migration",
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
    /// Write the complete canonical `migration.intent.next`.
    MigrationWriteIntentStage,
    /// Synchronize `migration.intent.next`.
    MigrationSynchronizeIntentStage,
    /// Link the synchronized intent stage to `migration.intent`.
    MigrationLinkIntent,
    /// Synchronize the store root after the intent link.
    MigrationSynchronizeRootAfterIntent,
    /// Remove the retained `migration.intent.next`.
    MigrationRemoveIntentStage,
    /// Synchronize the store root after intent-stage cleanup.
    MigrationSynchronizeRootAfterIntentCleanup,
    /// Create or exactly admit the persistent reader fence.
    MigrationAdmitReaderFence,
    /// Create or exactly admit the canonical version-2 directory prefix; the
    /// occurrence names the directory-prefix length reached.
    MigrationAdmitNamespacePrefix,
    /// Synchronize the store root after namespace admission.
    MigrationSynchronizeRootAfterNamespace,
    /// Write the complete canonical `FORMAT.next`.
    MigrationWriteMarkerStage,
    /// Synchronize `FORMAT.next`.
    MigrationSynchronizeMarkerStage,
    /// Link the synchronized marker stage to `FORMAT`.
    MigrationLinkMarker,
    /// Synchronize the store root after the marker link.
    MigrationSynchronizeRootAfterMarker,
    /// Remove the retained `FORMAT.next`.
    MigrationRemoveMarkerStage,
    /// Synchronize the store root after marker-stage cleanup.
    MigrationSynchronizeRootAfterMarkerCleanup,
    /// Write the complete canonical `migration.receipt.next`.
    MigrationWriteReceiptStage,
    /// Synchronize `migration.receipt.next`.
    MigrationSynchronizeReceiptStage,
    /// Link the synchronized receipt stage to `migration.receipt`.
    MigrationLinkReceipt,
    /// Synchronize the store root after the receipt link.
    MigrationSynchronizeRootAfterReceipt,
    /// Remove the retained `migration.receipt.next`.
    MigrationRemoveReceiptStage,
    /// Synchronize the store root after receipt-stage cleanup.
    MigrationSynchronizeRootAfterReceiptCleanup,
}

impl DurabilityCrashPoint {
    /// Every crash boundary in stable protocol order.
    pub const ALL: [Self; 56] = [
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
        Self::MigrationWriteIntentStage,
        Self::MigrationSynchronizeIntentStage,
        Self::MigrationLinkIntent,
        Self::MigrationSynchronizeRootAfterIntent,
        Self::MigrationRemoveIntentStage,
        Self::MigrationSynchronizeRootAfterIntentCleanup,
        Self::MigrationAdmitReaderFence,
        Self::MigrationAdmitNamespacePrefix,
        Self::MigrationSynchronizeRootAfterNamespace,
        Self::MigrationWriteMarkerStage,
        Self::MigrationSynchronizeMarkerStage,
        Self::MigrationLinkMarker,
        Self::MigrationSynchronizeRootAfterMarker,
        Self::MigrationRemoveMarkerStage,
        Self::MigrationSynchronizeRootAfterMarkerCleanup,
        Self::MigrationWriteReceiptStage,
        Self::MigrationSynchronizeReceiptStage,
        Self::MigrationLinkReceipt,
        Self::MigrationSynchronizeRootAfterReceipt,
        Self::MigrationRemoveReceiptStage,
        Self::MigrationSynchronizeRootAfterReceiptCleanup,
    ];

    /// The migration boundaries in `StoreMigrationPhase::ALL` order.
    pub const MIGRATION: [Self; 21] = [
        Self::MigrationWriteIntentStage,
        Self::MigrationSynchronizeIntentStage,
        Self::MigrationLinkIntent,
        Self::MigrationSynchronizeRootAfterIntent,
        Self::MigrationRemoveIntentStage,
        Self::MigrationSynchronizeRootAfterIntentCleanup,
        Self::MigrationAdmitReaderFence,
        Self::MigrationAdmitNamespacePrefix,
        Self::MigrationSynchronizeRootAfterNamespace,
        Self::MigrationWriteMarkerStage,
        Self::MigrationSynchronizeMarkerStage,
        Self::MigrationLinkMarker,
        Self::MigrationSynchronizeRootAfterMarker,
        Self::MigrationRemoveMarkerStage,
        Self::MigrationSynchronizeRootAfterMarkerCleanup,
        Self::MigrationWriteReceiptStage,
        Self::MigrationSynchronizeReceiptStage,
        Self::MigrationLinkReceipt,
        Self::MigrationSynchronizeRootAfterReceipt,
        Self::MigrationRemoveReceiptStage,
        Self::MigrationSynchronizeRootAfterReceiptCleanup,
    ];

    /// The number of directories the migration namespace prefix admits; each
    /// is one `during` occurrence of
    /// [`Self::MigrationAdmitNamespacePrefix`].
    pub const NAMESPACE_PREFIX_DIRECTORIES: u32 = 6;

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
            Self::MigrationWriteIntentStage
            | Self::MigrationSynchronizeIntentStage
            | Self::MigrationLinkIntent
            | Self::MigrationSynchronizeRootAfterIntent
            | Self::MigrationRemoveIntentStage
            | Self::MigrationSynchronizeRootAfterIntentCleanup
            | Self::MigrationAdmitReaderFence
            | Self::MigrationAdmitNamespacePrefix
            | Self::MigrationSynchronizeRootAfterNamespace
            | Self::MigrationWriteMarkerStage
            | Self::MigrationSynchronizeMarkerStage
            | Self::MigrationLinkMarker
            | Self::MigrationSynchronizeRootAfterMarker
            | Self::MigrationRemoveMarkerStage
            | Self::MigrationSynchronizeRootAfterMarkerCleanup
            | Self::MigrationWriteReceiptStage
            | Self::MigrationSynchronizeReceiptStage
            | Self::MigrationLinkReceipt
            | Self::MigrationSynchronizeRootAfterReceipt
            | Self::MigrationRemoveReceiptStage
            | Self::MigrationSynchronizeRootAfterReceiptCleanup => {
                DurabilityCrashSequence::Migration
            }
        }
    }

    /// Reports whether tests may select a repeated occurrence.
    #[must_use]
    pub const fn occurrence_counted(self) -> bool {
        matches!(
            self,
            Self::AppendSegmentRecord | Self::MigrationAdmitNamespacePrefix
        )
    }

    /// Returns how many distinct `during` occurrences the canonical matrix
    /// runs for this boundary: one directory-prefix length per occurrence
    /// for the migration namespace prefix, otherwise exactly one.
    #[must_use]
    pub const fn during_occurrences(self) -> u32 {
        match self {
            Self::MigrationAdmitNamespacePrefix => Self::NAMESPACE_PREFIX_DIRECTORIES,
            _ => 1,
        }
    }
}
