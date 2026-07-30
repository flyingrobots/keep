//! This module owns complete filesystem migration catalog-pool admission.

use std::collections::TryReserveError;

use cap_std::fs::Dir;

use super::StoreMigrationInventoryEntry;
use super::filesystem_inventory_catalog_errors;
use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, MigrationInventoryPool,
};
use super::filesystem_inventory_file::{
    self, FilesystemInventoryFileError, FilesystemInventoryFilePolicy,
};
use super::filesystem_inventory_names;
use super::filesystem_inventory_segments::FilesystemMigrationSegmentInventory;
use super::migration_catalog_admission::{self, MigrationSegmentLoadError};
use crate::CatalogLength;
use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase, ChecksummedCatalog,
    RecoveryEntryName, SegmentDigest, SegmentReadPolicy, physical_pool_name, recovery_pool_name,
};

const POOL: MigrationInventoryPool = MigrationInventoryPool::Catalogs;

pub(super) struct FilesystemMigrationCatalogInventory {
    entries: Vec<StoreMigrationInventoryEntry>,
    names: Vec<RecoveryEntryName>,
    remaining: u32,
}

impl FilesystemMigrationCatalogInventory {
    pub(super) fn entries(&self) -> &[StoreMigrationInventoryEntry] {
        &self.entries
    }

    pub(super) const fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn verify_names(
        &self,
        directory: &Dir,
    ) -> Result<(), FilesystemMigrationInventoryError> {
        filesystem_inventory_names::verify(directory, POOL, self.remaining, &self.names)
    }
}

pub(super) fn read(
    catalogs: &Dir,
    segments: &Dir,
    admitted_segments: &FilesystemMigrationSegmentInventory,
    remaining: u32,
    policy: SegmentReadPolicy,
) -> Result<FilesystemMigrationCatalogInventory, FilesystemMigrationInventoryError> {
    let names = filesystem_inventory_names::read(catalogs, POOL, remaining)?;
    let capacity = names.len();
    let entry_count = u64::try_from(capacity)
        .map_err(|_source| FilesystemMigrationInventoryError::EntryCountHostWidth { pool: POOL })?;
    let mut entries = reserve(capacity, entry_count)?;
    for name in &names {
        entries.push(admit(catalogs, segments, admitted_segments, name, policy)?);
    }
    entries.sort_unstable();
    Ok(FilesystemMigrationCatalogInventory {
        entries,
        names,
        remaining,
    })
}

fn reserve<T>(
    capacity: usize,
    entry_count: u64,
) -> Result<Vec<T>, FilesystemMigrationInventoryError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|source| allocation(entry_count, source))?;
    Ok(values)
}

const fn allocation(
    entry_count: u64,
    source: TryReserveError,
) -> FilesystemMigrationInventoryError {
    FilesystemMigrationInventoryError::Allocation {
        pool: POOL,
        entry_count,
        source,
    }
}

fn admit(
    catalogs: &Dir,
    segments: &Dir,
    admitted_segments: &FilesystemMigrationSegmentInventory,
    name: &RecoveryEntryName,
    policy: SegmentReadPolicy,
) -> Result<StoreMigrationInventoryEntry, FilesystemMigrationInventoryError> {
    let (generation, digest) = recovery_pool_name::catalog(name).map_err(|source| {
        FilesystemMigrationInventoryError::Name {
            pool: POOL,
            name: name.clone(),
            source,
        }
    })?;
    let encoded = read_catalog(catalogs, name, generation, digest)?;
    let catalog = ChecksummedCatalog::decode(&encoded).map_err(|source| {
        filesystem_inventory_catalog_errors::artifact(name, CatalogRestartError::Catalog { source })
    })?;
    require_coordinate(name, generation, digest, catalog)?;
    let admitted = migration_catalog_admission::admit(catalog, policy, |required| {
        load_segment(segments, admitted_segments, required)
    })
    .map_err(|source| filesystem_inventory_catalog_errors::admission(name, source))?;
    Ok(StoreMigrationInventoryEntry::from_migration_catalog(
        &admitted,
    ))
}

fn read_catalog(
    directory: &Dir,
    name: &RecoveryEntryName,
    generation: crate::CatalogGeneration,
    digest: crate::CatalogDigest,
) -> Result<Vec<u8>, FilesystemMigrationInventoryError> {
    let canonical_name = physical_pool_name::catalog(generation, digest);
    let policy = FilesystemInventoryFilePolicy::new(
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::OpenCatalog,
        CatalogRestartPhase::ReadCatalog,
        CatalogLength::MAXIMUM.get(),
    );
    filesystem_inventory_file::read(directory, &canonical_name, policy)
        .map_err(|source| filesystem_inventory_catalog_errors::catalog_file(name, source))
}

fn require_coordinate(
    name: &RecoveryEntryName,
    generation: crate::CatalogGeneration,
    digest: crate::CatalogDigest,
    catalog: ChecksummedCatalog<'_>,
) -> Result<(), FilesystemMigrationInventoryError> {
    if catalog.generation() == generation && catalog.digest() == digest {
        return Ok(());
    }
    Err(filesystem_inventory_catalog_errors::artifact(
        name,
        CatalogRestartError::CatalogCoordinate {
            expected_generation: generation,
            observed_generation: catalog.generation(),
            expected_length: catalog.length(),
            observed_length: catalog.length(),
            expected_digest: digest,
            observed_digest: catalog.digest(),
        },
    ))
}

fn load_segment(
    directory: &Dir,
    admitted: &FilesystemMigrationSegmentInventory,
    digest: SegmentDigest,
) -> Result<Vec<u8>, MigrationSegmentLoadError<FilesystemInventoryFileError>> {
    if !admitted.contains(digest) {
        return Err(MigrationSegmentLoadError::Missing);
    }
    let name = physical_pool_name::segment(digest);
    let policy = FilesystemInventoryFilePolicy::new(
        CatalogRestartArtifact::Segment { digest },
        CatalogRestartPhase::OpenSegment,
        CatalogRestartPhase::ReadSegment,
        crate::adapters::segment_header::MAXIMUM_SEGMENT_LENGTH,
    );
    filesystem_inventory_file::read(directory, &name, policy)
        .map_err(MigrationSegmentLoadError::Source)
}
