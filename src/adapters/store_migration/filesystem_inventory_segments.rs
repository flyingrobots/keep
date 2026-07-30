//! This module owns complete filesystem migration segment-pool admission.

use std::collections::TryReserveError;

use cap_std::fs::Dir;

use super::StoreMigrationInventoryEntry;
use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, MigrationInventoryPool,
};
use super::filesystem_inventory_file::{
    self, FilesystemInventoryFileError, FilesystemInventoryFilePolicy,
};
use super::filesystem_inventory_names;
use crate::adapters::segment_header::MAXIMUM_SEGMENT_LENGTH;
use crate::adapters::{
    AdmittedSegment, CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase,
    RecoveryEntryName, SegmentDigest, SegmentReadPolicy, physical_pool_name, recovery_pool_name,
};

const POOL: MigrationInventoryPool = MigrationInventoryPool::Segments;
pub(super) struct FilesystemMigrationSegmentInventory {
    entries: Vec<StoreMigrationInventoryEntry>,
    digests: Vec<SegmentDigest>,
    names: Vec<RecoveryEntryName>,
    remaining: u32,
}

impl FilesystemMigrationSegmentInventory {
    pub(super) fn entries(&self) -> &[StoreMigrationInventoryEntry] {
        &self.entries
    }

    pub(super) fn contains(&self, digest: SegmentDigest) -> bool {
        self.digests.binary_search(&digest).is_ok()
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
    directory: &Dir,
    remaining: u32,
    policy: SegmentReadPolicy,
) -> Result<FilesystemMigrationSegmentInventory, FilesystemMigrationInventoryError> {
    let names = filesystem_inventory_names::read(directory, POOL, remaining)?;
    let capacity = names.len();
    let entry_count = u64::try_from(capacity)
        .map_err(|_source| FilesystemMigrationInventoryError::EntryCountHostWidth { pool: POOL })?;
    let mut entries = reserve(capacity, entry_count)?;
    let mut digests = reserve(capacity, entry_count)?;
    for name in &names {
        let (entry, digest) = admit(directory, name, policy)?;
        entries.push(entry);
        digests.push(digest);
    }
    entries.sort_unstable();
    digests.sort_unstable();
    Ok(FilesystemMigrationSegmentInventory {
        entries,
        digests,
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
    directory: &Dir,
    name: &RecoveryEntryName,
    policy: SegmentReadPolicy,
) -> Result<(StoreMigrationInventoryEntry, SegmentDigest), FilesystemMigrationInventoryError> {
    let expected = parse_name(name)?;
    let encoded = read_encoded(directory, name, expected)?;
    let segment = AdmittedSegment::decode(&encoded, policy).map_err(|source| {
        artifact_error(
            name,
            CatalogRestartError::Segment {
                expected,
                source: Box::new(source),
            },
        )
    })?;
    if segment.digest() != expected {
        return Err(artifact_error(
            name,
            CatalogRestartError::SegmentCoordinate {
                expected,
                observed: segment.digest(),
            },
        ));
    }
    Ok((
        StoreMigrationInventoryEntry::from_segment(&segment),
        segment.digest(),
    ))
}

fn parse_name(
    name: &RecoveryEntryName,
) -> Result<SegmentDigest, FilesystemMigrationInventoryError> {
    recovery_pool_name::segment(name).map_err(|source| FilesystemMigrationInventoryError::Name {
        pool: POOL,
        name: name.clone(),
        source,
    })
}

fn read_encoded(
    directory: &Dir,
    name: &RecoveryEntryName,
    expected: SegmentDigest,
) -> Result<Vec<u8>, FilesystemMigrationInventoryError> {
    let canonical_name = physical_pool_name::segment(expected);
    let artifact = CatalogRestartArtifact::Segment { digest: expected };
    filesystem_inventory_file::read(
        directory,
        &canonical_name,
        FilesystemInventoryFilePolicy::new(
            artifact,
            CatalogRestartPhase::OpenSegment,
            CatalogRestartPhase::ReadSegment,
            MAXIMUM_SEGMENT_LENGTH,
        ),
    )
    .map_err(|source| file_error(name, source))
}

fn artifact_error(
    name: &RecoveryEntryName,
    source: CatalogRestartError,
) -> FilesystemMigrationInventoryError {
    FilesystemMigrationInventoryError::Artifact {
        pool: POOL,
        name: name.clone(),
        source: Box::new(source),
    }
}

fn file_error(
    name: &RecoveryEntryName,
    source: FilesystemInventoryFileError,
) -> FilesystemMigrationInventoryError {
    match source {
        FilesystemInventoryFileError::Artifact(source) => artifact_error(name, *source),
        FilesystemInventoryFileError::Changed => {
            FilesystemMigrationInventoryError::ArtifactChanged {
                pool: POOL,
                name: name.clone(),
            }
        }
    }
}
