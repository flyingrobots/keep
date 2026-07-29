//! This module owns required initialized recovery-root entries.

use std::fmt;

/// One fixed entry required in every initialized store root.
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
}

impl fmt::Display for RecoveryRequiredEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WriterLock => "writer.lock",
            Self::StagingDirectory => "staging",
            Self::SegmentPoolDirectory => "segments",
            Self::CatalogPoolDirectory => "catalogs",
        })
    }
}
