//! This module owns semantic recovery namespace roles.

use super::SegmentDigest;
use crate::{CatalogDigest, CatalogGeneration};

/// Semantic role selected by one canonical recovery entry name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryEntryRole {
    /// Persistent writer-lock file.
    WriterLock,
    /// Canonical staging directory.
    StagingDirectory,
    /// Canonical immutable segment-pool directory.
    SegmentPoolDirectory,
    /// Canonical immutable catalog-pool directory.
    CatalogPoolDirectory,
    /// Current publication head.
    CurrentHead,
    /// Candidate next publication head.
    NextHeadStage,
    /// Fixed segment staging file.
    SegmentStage,
    /// Fixed catalog staging file.
    CatalogStage,
    /// Digest-addressed immutable segment.
    ImmutableSegment {
        /// Physical segment digest parsed from the name.
        digest: SegmentDigest,
    },
    /// Generation-and-digest-addressed immutable catalog.
    ImmutableCatalog {
        /// Catalog generation parsed from the name.
        generation: CatalogGeneration,
        /// Physical catalog digest parsed from the name.
        digest: CatalogDigest,
    },
    /// Version-two persistent reader fence `reader.lock`.
    ReaderFence,
    /// Version-two format marker `FORMAT`.
    FormatMarker,
    /// Version-two migration intent record.
    MigrationIntent,
    /// Version-two migration receipt record.
    MigrationReceipt,
    /// Version-two `retention` protocol directory, admitted by retention recovery.
    RetentionDirectory,
    /// Version-two reserved `gc` protocol directory.
    GcDirectory,
    /// Version-two reserved `recovery` protocol directory.
    RecoveryDirectory,
}

impl RecoveryEntryRole {
    pub(in crate::adapters) const fn is_stage(self) -> bool {
        matches!(
            self,
            Self::NextHeadStage | Self::SegmentStage | Self::CatalogStage
        )
    }
}
