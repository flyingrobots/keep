//! This module owns stable text identities for durability crash points.

use crate::durability_crash_point::DurabilityCrashPoint;

impl DurabilityCrashPoint {
    /// Returns the stable `KEEP-CRASH-NNN` identifier.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::CreateSegmentStage => "KEEP-CRASH-001",
            Self::WriteSegmentHeader => "KEEP-CRASH-002",
            Self::AppendSegmentRecord => "KEEP-CRASH-003",
            Self::FlushSegmentRecordPrefix => "KEEP-CRASH-004",
            Self::SynchronizeSegmentRecordPrefix => "KEEP-CRASH-005",
            Self::AppendSegmentSeal => "KEEP-CRASH-006",
            Self::FlushSealedSegment => "KEEP-CRASH-007",
            Self::SynchronizeSealedSegment => "KEEP-CRASH-008",
            Self::LinkSegment => "KEEP-CRASH-009",
            Self::SynchronizeSegmentPool => "KEEP-CRASH-010",
            Self::RemoveSegmentStage => "KEEP-CRASH-011",
            Self::SynchronizeStagingAfterSegment => "KEEP-CRASH-012",
            Self::CreateCatalogStage => "KEEP-CRASH-013",
            Self::WriteCatalog => "KEEP-CRASH-014",
            Self::FlushCatalog => "KEEP-CRASH-015",
            Self::SynchronizeCatalog => "KEEP-CRASH-016",
            Self::LinkCatalog => "KEEP-CRASH-017",
            Self::SynchronizeCatalogPool => "KEEP-CRASH-018",
            Self::RemoveCatalogStage => "KEEP-CRASH-019",
            Self::SynchronizeStagingAfterCatalog => "KEEP-CRASH-020",
            Self::CreateHeadStage => "KEEP-CRASH-021",
            Self::WriteHead => "KEEP-CRASH-022",
            Self::FlushHead => "KEEP-CRASH-023",
            Self::SynchronizeHead => "KEEP-CRASH-024",
            Self::ReplaceHead => "KEEP-CRASH-025",
            Self::SynchronizeRootAfterHead => "KEEP-CRASH-026",
            Self::RemoveRecoveryStage => "KEEP-CRASH-027",
            Self::SynchronizeStagingAfterRecovery => "KEEP-CRASH-028",
            Self::RemoveRecoveryHead => "KEEP-CRASH-029",
            Self::SynchronizeRootAfterRecovery => "KEEP-CRASH-030",
            Self::OpenAndLockWriterFile => "KEEP-CRASH-031",
            Self::CreateStagingDirectory => "KEEP-CRASH-032",
            Self::CreateSegmentPoolDirectory => "KEEP-CRASH-033",
            Self::CreateCatalogPoolDirectory => "KEEP-CRASH-034",
            Self::SynchronizeRootAfterInitialization => "KEEP-CRASH-035",
        }
    }
}
