//! This module owns required initialized recovery-root entries.

use std::fmt;

/// One fixed entry required in an initialized store root.
///
/// The first four are required in every root. The remaining entries are
/// required only when a migrated version-two root is classified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryRequiredEntry {
    /// Persistent `writer.lock`.
    WriterLock,
    /// `staging` directory.
    StagingDirectory,
    /// `segments` directory.
    SegmentPoolDirectory,
    /// `catalogs` directory.
    CatalogPoolDirectory,
    /// Published catalog `HEAD`, required in a version-two root.
    CurrentHead,
    /// Persistent `reader.lock`.
    ReaderFence,
    /// `FORMAT` marker.
    FormatMarker,
    /// `migration.intent` record.
    MigrationIntent,
    /// `migration.receipt` record.
    MigrationReceipt,
    /// `retention` directory.
    RetentionDirectory,
    /// `gc` directory.
    GcDirectory,
    /// `recovery` directory.
    RecoveryDirectory,
}

impl fmt::Display for RecoveryRequiredEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WriterLock => "writer.lock",
            Self::StagingDirectory => "staging",
            Self::SegmentPoolDirectory => "segments",
            Self::CatalogPoolDirectory => "catalogs",
            Self::CurrentHead => "HEAD",
            Self::ReaderFence => "reader.lock",
            Self::FormatMarker => "FORMAT",
            Self::MigrationIntent => "migration.intent",
            Self::MigrationReceipt => "migration.receipt",
            Self::RetentionDirectory => "retention",
            Self::GcDirectory => "gc",
            Self::RecoveryDirectory => "recovery",
        })
    }
}
