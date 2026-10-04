//! This module owns migration-authority coordinate validation.

use super::filesystem_inventory_file::FilesystemInventoryFileError;
use super::filesystem_migration_authority_error::{
    FilesystemMigrationAuthorityArtifact as Artifact, FilesystemMigrationAuthorityError as Error,
    StoreRootIdentityCoordinate as RootCoordinate,
};
use super::migration_catalog_coordinates::MigrationCatalogCoordinates;
use crate::adapters::{ChecksummedCatalog, ChecksummedPublicationHead};

pub(super) const fn require_root(
    coordinate: RootCoordinate,
    expected: u64,
    observed: u64,
) -> Result<(), Error> {
    if expected == observed {
        Ok(())
    } else {
        Err(Error::RootIdentityChanged {
            coordinate,
            expected,
            observed,
        })
    }
}

pub(super) fn artifact_error(artifact: Artifact, source: FilesystemInventoryFileError) -> Error {
    match source {
        FilesystemInventoryFileError::Artifact(source) => Error::Artifact { artifact, source },
        FilesystemInventoryFileError::Changed => Error::ArtifactChanged { artifact },
    }
}

pub(super) fn verify_catalog(
    head: ChecksummedPublicationHead<'_>,
    catalog: ChecksummedCatalog<'_>,
) -> Result<MigrationCatalogCoordinates, Error> {
    require_generation(head.generation(), catalog.generation())?;
    require_length(head.catalog_length(), catalog.length())?;
    require_digest(head.catalog_digest(), catalog.digest())?;
    Ok(MigrationCatalogCoordinates::new(
        catalog.generation(),
        catalog.length(),
        catalog.digest(),
        catalog.previous_catalog_digest(),
    ))
}

fn require_generation(
    expected: crate::CatalogGeneration,
    observed: crate::CatalogGeneration,
) -> Result<(), Error> {
    if expected == observed {
        Ok(())
    } else {
        Err(Error::CatalogGeneration { expected, observed })
    }
}

fn require_length(
    expected: crate::CatalogLength,
    observed: crate::CatalogLength,
) -> Result<(), Error> {
    if expected == observed {
        Ok(())
    } else {
        Err(Error::CatalogLength { expected, observed })
    }
}

fn require_digest(
    expected: crate::CatalogDigest,
    observed: crate::CatalogDigest,
) -> Result<(), Error> {
    if expected == observed {
        Ok(())
    } else {
        Err(Error::CatalogDigest { expected, observed })
    }
}
