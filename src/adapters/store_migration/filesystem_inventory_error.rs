//! This boundary module owns filesystem migration-inventory failures.

use std::collections::TryReserveError;
use std::io;

use super::super::{CatalogRestartError, RecoveryEntryName, RecoveryPoolNameError, SegmentDigest};

/// Immutable version-1 pool selected during migration inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationInventoryPool {
    /// The `segments` immutable pool.
    Segments,
    /// The `catalogs` immutable pool.
    Catalogs,
}

/// Pinned filesystem namespace observed during migration inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationInventoryNamespace {
    /// The writer-authorized store root.
    Root,
    /// The `segments` immutable pool.
    Segments,
    /// The `catalogs` immutable pool.
    Catalogs,
}

impl From<MigrationInventoryPool> for MigrationInventoryNamespace {
    fn from(pool: MigrationInventoryPool) -> Self {
        match pool {
            MigrationInventoryPool::Segments => Self::Segments,
            MigrationInventoryPool::Catalogs => Self::Catalogs,
        }
    }
}

/// Capability-relative directory operation attempted during inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemMigrationInventoryOperation {
    /// Clone the writer-authorized root capability.
    CloneRoot,
    /// Open one immutable-pool directory without following links.
    OpenPool,
    /// Revalidate one pinned immutable-pool directory.
    VerifyPool,
    /// Count entries under the pinned directory capability.
    CountEntries,
    /// Read exact raw entry names under the pinned directory capability.
    ReadEntryNames,
}

/// Failure to derive exact migration inventory from immutable filesystem pools.
#[derive(Debug)]
pub enum FilesystemMigrationInventoryError {
    /// One capability-relative directory operation failed.
    Io {
        /// Namespace being observed.
        namespace: MigrationInventoryNamespace,
        /// Exact failed operation.
        operation: FilesystemMigrationInventoryOperation,
        /// Preserved filesystem source.
        source: io::Error,
    },
    /// A pinned immutable-pool directory changed identity.
    NamespaceChanged {
        /// Pool whose canonical directory entry changed.
        pool: MigrationInventoryPool,
    },
    /// Exact raw pool membership changed during artifact admission.
    EntriesChanged {
        /// Pool whose entry-name set changed.
        pool: MigrationInventoryPool,
    },
    /// A pool exceeded the remaining inventory entry budget.
    EntryLimitExceeded {
        /// Pool being observed.
        pool: MigrationInventoryPool,
        /// Remaining entry budget.
        maximum: u32,
        /// Smallest count observed before enumeration stopped.
        observed_at_least: u64,
    },
    /// The directory entry count changed between bounded passes.
    EntryCountChanged {
        /// Pool being observed.
        pool: MigrationInventoryPool,
        /// Count established by the first pass.
        expected: u64,
        /// Count established by the second pass.
        observed: u64,
    },
    /// A host entry count did not fit the protocol representation.
    EntryCountHostWidth {
        /// Pool being observed.
        pool: MigrationInventoryPool,
    },
    /// Memory could not retain the bounded semantic inventory.
    Allocation {
        /// Pool being observed.
        pool: MigrationInventoryPool,
        /// Exact number of entries requested.
        entry_count: u64,
        /// Preserved allocation source.
        source: TryReserveError,
    },
    /// One immutable-pool name was not canonical.
    Name {
        /// Pool containing the entry.
        pool: MigrationInventoryPool,
        /// Exact raw name that was refused.
        name: RecoveryEntryName,
        /// Preserved canonical-name refusal.
        source: RecoveryPoolNameError,
    },
    /// One named immutable artifact could not be completely admitted.
    Artifact {
        /// Pool containing the artifact.
        pool: MigrationInventoryPool,
        /// Exact raw canonical name.
        name: RecoveryEntryName,
        /// Preserved artifact refusal.
        source: Box<CatalogRestartError>,
    },
    /// A canonical artifact changed identity while it was being admitted.
    ArtifactChanged {
        /// Pool containing the artifact.
        pool: MigrationInventoryPool,
        /// Exact raw canonical name.
        name: RecoveryEntryName,
    },
    /// A catalog-referenced segment could not be completely admitted.
    ReferencedSegment {
        /// Exact segment coordinate required by the catalog.
        digest: SegmentDigest,
        /// Preserved segment artifact refusal.
        source: Box<CatalogRestartError>,
    },
    /// A catalog-referenced segment changed identity during admission.
    ReferencedSegmentChanged {
        /// Exact segment coordinate required by the catalog.
        digest: SegmentDigest,
    },
    /// Combined pool count arithmetic could not be represented.
    EntryCountArithmetic,
    /// Combined pool count violated the canonical inventory bound.
    EntryCount {
        /// Preserved canonical count refusal.
        source: super::StoreMigrationInventoryEntryCountError,
    },
    /// Canonical entry streaming refused the observed inventory.
    Canonical {
        /// Preserved canonical inventory refusal.
        source: Box<super::StoreMigrationInventoryError>,
    },
}
