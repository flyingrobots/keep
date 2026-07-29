//! This module owns exact segment-store initialization phases.

use std::fmt;

/// Exact operation attempted during store initialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreInitializationPhase {
    /// Prove the platform contract before namespace mutation.
    AdmitPlatform,
    /// Create or reopen `writer.lock`, then acquire writer authority.
    OpenAndLockWriterFile,
    /// Create or verify the `staging` directory.
    AdmitStagingDirectory,
    /// Create or verify the `segments` directory.
    AdmitSegmentPoolDirectory,
    /// Create or verify the `catalogs` directory.
    AdmitCatalogPoolDirectory,
    /// Synchronize the store root after all canonical names exist.
    SynchronizeRoot,
}

impl fmt::Display for StoreInitializationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AdmitPlatform => "platform admission",
            Self::OpenAndLockWriterFile => "writer-lock admission",
            Self::AdmitStagingDirectory => "staging-directory admission",
            Self::AdmitSegmentPoolDirectory => "segment-pool admission",
            Self::AdmitCatalogPoolDirectory => "catalog-pool admission",
            Self::SynchronizeRoot => "root synchronization",
        })
    }
}
