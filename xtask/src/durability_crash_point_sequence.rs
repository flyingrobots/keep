//! This module owns the sequence each durability crash point belongs to.

use crate::durability_crash_point::{DurabilityCrashPoint, DurabilityCrashSequence};

impl DurabilityCrashPoint {
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
            Self::GcWriteIntentStage
            | Self::GcSynchronizeIntentStage
            | Self::GcLinkIntent
            | Self::GcSynchronizeGcAfterIntent
            | Self::GcRemoveIntentStage
            | Self::GcSynchronizeGcAfterIntentCleanup
            | Self::GcUnlinkCandidate
            | Self::GcSynchronizeSegmentPool
            | Self::GcWriteReceiptStage
            | Self::GcSynchronizeReceiptStage
            | Self::GcReplaceReceipt
            | Self::GcSynchronizeGcAfterReceipt
            | Self::GcRemoveIntent
            | Self::GcSynchronizeGcAfterIntentRemoval => DurabilityCrashSequence::Gc,
        }
    }
}
