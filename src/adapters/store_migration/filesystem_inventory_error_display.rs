//! This module owns filesystem migration-inventory error presentation.

use std::error::Error;
use std::fmt;

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};

impl fmt::Display for MigrationInventoryPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Segments => "segments",
            Self::Catalogs => "catalogs",
        })
    }
}

impl fmt::Display for FilesystemMigrationInventoryOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CloneRoot => "clone writer-authorized root",
            Self::OpenPool => "open pool capability",
            Self::VerifyPool => "verify pool capability",
            Self::CountEntries => "count entries",
            Self::ReadEntryNames => "read entry names",
        })
    }
}

impl fmt::Display for MigrationInventoryNamespace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Root => "root",
            Self::Segments => "segments",
            Self::Catalogs => "catalogs",
        })
    }
}

impl fmt::Display for FilesystemMigrationInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                namespace,
                operation,
                ..
            } => write!(
                formatter,
                "migration inventory failed to {operation} in {namespace}"
            ),
            Self::NamespaceChanged { pool } => {
                write!(
                    formatter,
                    "migration inventory namespace changed for {pool}"
                )
            }
            Self::EntriesChanged { pool } => {
                write!(formatter, "migration inventory entries changed for {pool}")
            }
            Self::EntryLimitExceeded {
                pool,
                maximum,
                observed_at_least,
            } => write!(
                formatter,
                "migration inventory observed at least {observed_at_least} entries in {pool}, \
                 exceeding remaining limit {maximum}"
            ),
            Self::EntryCountChanged {
                pool,
                expected,
                observed,
            } => write!(
                formatter,
                "migration inventory entry count changed in {pool}: expected {expected}, \
                 observed {observed}"
            ),
            Self::EntryCountHostWidth { pool } => {
                write!(
                    formatter,
                    "migration inventory count does not fit for {pool}"
                )
            }
            Self::Allocation {
                pool, entry_count, ..
            } => write!(
                formatter,
                "migration inventory could not retain {entry_count} entries for {pool}"
            ),
            Self::Name { pool, name, .. } => write!(
                formatter,
                "migration inventory found noncanonical name {:?} in {pool}",
                name.as_bytes()
            ),
            Self::Artifact { pool, name, .. } => write!(
                formatter,
                "migration inventory could not admit artifact {:?} in {pool}",
                name.as_bytes()
            ),
            Self::ArtifactChanged { pool, name } => write!(
                formatter,
                "migration inventory artifact {:?} changed identity in {pool}",
                name.as_bytes()
            ),
            Self::ReferencedSegment { digest, .. } => write!(
                formatter,
                "migration inventory could not admit catalog segment {digest:?}"
            ),
            Self::ReferencedSegmentChanged { digest } => write!(
                formatter,
                "migration inventory catalog segment {digest:?} changed identity"
            ),
            Self::EntryCountArithmetic => {
                formatter.write_str("migration inventory entry count overflowed")
            }
            Self::EntryCount { .. } => {
                formatter.write_str("migration inventory entry count was refused")
            }
            Self::Canonical { .. } => {
                formatter.write_str("migration inventory canonical streaming failed")
            }
        }
    }
}

impl Error for FilesystemMigrationInventoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::Name { source, .. } => Some(source),
            Self::Artifact { source, .. } | Self::ReferencedSegment { source, .. } => Some(source),
            Self::EntryCount { source } => Some(source),
            Self::Canonical { source } => Some(source),
            Self::EntryLimitExceeded { .. }
            | Self::EntryCountChanged { .. }
            | Self::EntryCountHostWidth { .. }
            | Self::ArtifactChanged { .. }
            | Self::ReferencedSegmentChanged { .. }
            | Self::NamespaceChanged { .. }
            | Self::EntriesChanged { .. }
            | Self::EntryCountArithmetic => None,
        }
    }
}
