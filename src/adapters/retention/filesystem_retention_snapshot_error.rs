//! This module owns the typed error of filesystem retention snapshot loading.

use std::error::Error;
use std::fmt;
use std::io;

use super::RetentionViewError;
use crate::adapters::CatalogRestartError;

/// Why a reader could not bind one consistent version-two view.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemRetentionSnapshotError {
    /// The platform profile or exact version-two root admission failed.
    Admission {
        /// The exact namespace or record refusal.
        source: io::Error,
    },
    /// The reader fence could not be acquired.
    Fence {
        /// The exact filesystem failure.
        source: io::Error,
    },
    /// The heads never agreed, or a coordinate or retention read failed.
    View {
        /// The exact collection refusal.
        source: RetentionViewError,
    },
    /// Stable collected heads select a catalog that does not admit.
    Catalog {
        /// The exact restart refusal.
        source: CatalogRestartError,
    },
    /// A selected root pool entry is absent, unreadable, or not the manifest's.
    Root {
        /// The exact filesystem or decode refusal.
        source: io::Error,
    },
}

impl fmt::Display for FilesystemRetentionSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Admission { .. } => "version-two reader admission refused",
            Self::Fence { .. } => "reader fence acquisition failed",
            Self::View { .. } => "reader view collection refused",
            Self::Catalog { .. } => "catalog snapshot refused",
            Self::Root { .. } => "selected retention root refused",
        })
    }
}

impl Error for FilesystemRetentionSnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission { source } | Self::Fence { source } | Self::Root { source } => {
                Some(source)
            }
            Self::View { source } => Some(source),
            Self::Catalog { source } => Some(source),
        }
    }
}
