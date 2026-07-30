//! This module owns writer-locked filesystem migration pool inventory.

use cap_std::fs::Dir;

use super::filesystem_inventory_catalogs;
use super::filesystem_inventory_catalogs::FilesystemMigrationCatalogInventory;
use super::filesystem_inventory_directory::PinnedMigrationPoolDirectory;
use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};
use super::filesystem_inventory_segments;
use super::filesystem_inventory_segments::FilesystemMigrationSegmentInventory;
use super::{
    ImmutablePoolInventoryDigest, StoreMigrationInventoryEntryCount, StoreMigrationInventoryHasher,
};
use crate::adapters::filesystem_root_identity::FilesystemRootIdentity;
use crate::adapters::{FilesystemPlatformAdmission, FilesystemWriterLock, SegmentReadPolicy};

const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

/// Writer-authorized reader for one exact version-1 immutable-pool inventory.
///
/// The reader retains the writer lock and pinned root, segment-pool, and
/// catalog-pool capabilities. It performs no protocol mutation.
#[must_use]
pub struct FilesystemStoreMigrationInventoryReader {
    root: Dir,
    segments: PinnedMigrationPoolDirectory,
    catalogs: PinnedMigrationPoolDirectory,
    policy: SegmentReadPolicy,
    root_identity: FilesystemRootIdentity,
    _lock: FilesystemWriterLock,
}

impl FilesystemStoreMigrationInventoryReader {
    /// Pins both immutable pools under admitted exclusive writer authority.
    ///
    /// The synchronous call performs bounded capability-relative filesystem
    /// I/O and allocates no content-sized memory.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationInventoryError`] when the root capability
    /// cannot be cloned or either canonical pool is missing, linked, replaced,
    /// or not a directory.
    pub fn open(
        admission: FilesystemPlatformAdmission,
        policy: SegmentReadPolicy,
    ) -> Result<Self, FilesystemMigrationInventoryError> {
        let (lock, root_identity) = admission.into_parts();
        let root =
            lock.clone_directory()
                .map_err(|source| FilesystemMigrationInventoryError::Io {
                    namespace: MigrationInventoryNamespace::Root,
                    operation: FilesystemMigrationInventoryOperation::CloneRoot,
                    source,
                })?;
        let segments = PinnedMigrationPoolDirectory::open(
            &root,
            MigrationInventoryPool::Segments,
            SEGMENTS_NAME,
        )?;
        let catalogs = PinnedMigrationPoolDirectory::open(
            &root,
            MigrationInventoryPool::Catalogs,
            CATALOGS_NAME,
        )?;
        Ok(Self {
            root,
            segments,
            catalogs,
            policy,
            root_identity,
            _lock: lock,
        })
    }

    /// Derives the exact bounded canonical inventory digest.
    ///
    /// Every segment and catalog pool entry is named canonically, opened
    /// without following links, read under the fixed format bound, verified
    /// against its physical coordinate, and completely admitted. Catalog
    /// record bindings reopen only referenced admitted segment coordinates, so
    /// peak content allocation is bounded by one catalog and one segment.
    ///
    /// The synchronous call may block on filesystem I/O and performs no
    /// protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationInventoryError`] for namespace drift,
    /// count drift or overflow, noncanonical names, linked or changed
    /// artifacts, malformed content, catalog binding failure, allocation
    /// refusal, or canonical digest-stream refusal.
    pub fn read(&self) -> Result<ImmutablePoolInventoryDigest, FilesystemMigrationInventoryError> {
        self.verify_directories()?;
        let (segments, catalogs) = self.read_pools()?;
        let digest = hash_inventory(&segments, &catalogs)?;
        segments.verify_names(self.segments.directory())?;
        catalogs.verify_names(self.catalogs.directory())?;
        self.verify_directories()?;
        Ok(digest)
    }

    fn read_pools(
        &self,
    ) -> Result<
        (
            FilesystemMigrationSegmentInventory,
            FilesystemMigrationCatalogInventory,
        ),
        FilesystemMigrationInventoryError,
    > {
        let maximum = StoreMigrationInventoryEntryCount::MAXIMUM;
        let segments =
            filesystem_inventory_segments::read(self.segments.directory(), maximum, self.policy)?;
        let segment_count = host_count(segments.len(), MigrationInventoryPool::Segments)?;
        let remaining = maximum
            .checked_sub(segment_count)
            .ok_or(FilesystemMigrationInventoryError::EntryCountArithmetic)?;
        let catalogs = filesystem_inventory_catalogs::read(
            self.catalogs.directory(),
            self.segments.directory(),
            &segments,
            remaining,
            self.policy,
        )?;
        Ok((segments, catalogs))
    }

    fn verify_directories(&self) -> Result<(), FilesystemMigrationInventoryError> {
        self.segments.verify(&self.root)?;
        self.catalogs.verify(&self.root)
    }

    pub(super) const fn root(&self) -> &Dir {
        &self.root
    }

    pub(super) const fn catalogs(&self) -> &Dir {
        self.catalogs.directory()
    }

    pub(super) const fn root_identity(&self) -> FilesystemRootIdentity {
        self.root_identity
    }
}

fn hash_inventory(
    segments: &FilesystemMigrationSegmentInventory,
    catalogs: &FilesystemMigrationCatalogInventory,
) -> Result<ImmutablePoolInventoryDigest, FilesystemMigrationInventoryError> {
    let segment_count = host_count(segments.len(), MigrationInventoryPool::Segments)?;
    let catalog_count = host_count(catalogs.len(), MigrationInventoryPool::Catalogs)?;
    let total = segment_count
        .checked_add(catalog_count)
        .ok_or(FilesystemMigrationInventoryError::EntryCountArithmetic)?;
    let count = StoreMigrationInventoryEntryCount::new(total)
        .map_err(|source| FilesystemMigrationInventoryError::EntryCount { source })?;
    let mut hasher = StoreMigrationInventoryHasher::new(count);
    for entry in segments.entries().iter().chain(catalogs.entries()) {
        hasher.push(*entry).map_err(canonical_error)?;
    }
    hasher.finish().map_err(canonical_error)
}

fn canonical_error(
    source: super::StoreMigrationInventoryError,
) -> FilesystemMigrationInventoryError {
    FilesystemMigrationInventoryError::Canonical {
        source: Box::new(source),
    }
}

fn host_count(
    count: usize,
    pool: MigrationInventoryPool,
) -> Result<u32, FilesystemMigrationInventoryError> {
    u32::try_from(count)
        .map_err(|_source| FilesystemMigrationInventoryError::EntryCountHostWidth { pool })
}
