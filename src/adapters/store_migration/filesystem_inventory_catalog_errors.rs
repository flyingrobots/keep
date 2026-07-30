//! This module owns filesystem migration catalog-inventory error translation.

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, MigrationInventoryPool,
};
use super::filesystem_inventory_file::FilesystemInventoryFileError;
use super::migration_catalog_admission::MigrationCatalogAdmissionError;
use crate::adapters::{
    CatalogAdmissionError, CatalogRestartError, RecoveryEntryName, SegmentDigest,
};

const POOL: MigrationInventoryPool = MigrationInventoryPool::Catalogs;

pub(super) fn admission(
    name: &RecoveryEntryName,
    source: MigrationCatalogAdmissionError<FilesystemInventoryFileError>,
) -> FilesystemMigrationInventoryError {
    match source {
        MigrationCatalogAdmissionError::Catalog(source) => catalog_admission(name, *source),
        MigrationCatalogAdmissionError::SegmentSource { digest, source } => {
            referenced_segment(digest, source)
        }
        MigrationCatalogAdmissionError::SegmentCoordinate { expected, observed } => {
            FilesystemMigrationInventoryError::ReferencedSegment {
                digest: expected,
                source: Box::new(CatalogRestartError::SegmentCoordinate { expected, observed }),
            }
        }
    }
}

fn catalog_admission(
    name: &RecoveryEntryName,
    source: CatalogAdmissionError,
) -> FilesystemMigrationInventoryError {
    match source {
        CatalogAdmissionError::MissingSegment { digest } => {
            FilesystemMigrationInventoryError::ReferencedSegment {
                digest,
                source: Box::new(CatalogRestartError::CatalogAdmission {
                    source: Box::new(CatalogAdmissionError::MissingSegment { digest }),
                }),
            }
        }
        CatalogAdmissionError::Segment { digest, source } => {
            FilesystemMigrationInventoryError::ReferencedSegment {
                digest,
                source: Box::new(CatalogRestartError::Segment {
                    expected: digest,
                    source,
                }),
            }
        }
        source => artifact(
            name,
            CatalogRestartError::CatalogAdmission {
                source: Box::new(source),
            },
        ),
    }
}

pub(super) fn catalog_file(
    name: &RecoveryEntryName,
    source: FilesystemInventoryFileError,
) -> FilesystemMigrationInventoryError {
    match source {
        FilesystemInventoryFileError::Artifact(source) => {
            FilesystemMigrationInventoryError::Artifact {
                pool: POOL,
                name: name.clone(),
                source,
            }
        }
        FilesystemInventoryFileError::Changed => {
            FilesystemMigrationInventoryError::ArtifactChanged {
                pool: POOL,
                name: name.clone(),
            }
        }
    }
}

fn referenced_segment(
    digest: SegmentDigest,
    source: FilesystemInventoryFileError,
) -> FilesystemMigrationInventoryError {
    match source {
        FilesystemInventoryFileError::Artifact(source) => {
            FilesystemMigrationInventoryError::ReferencedSegment { digest, source }
        }
        FilesystemInventoryFileError::Changed => {
            FilesystemMigrationInventoryError::ReferencedSegmentChanged { digest }
        }
    }
}

pub(super) fn artifact(
    name: &RecoveryEntryName,
    source: CatalogRestartError,
) -> FilesystemMigrationInventoryError {
    FilesystemMigrationInventoryError::Artifact {
        pool: POOL,
        name: name.clone(),
        source: Box::new(source),
    }
}
