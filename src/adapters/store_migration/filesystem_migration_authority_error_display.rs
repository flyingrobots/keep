//! This module owns filesystem migration-authority error presentation.

use std::error::Error;
use std::fmt;

use super::filesystem_migration_authority_error::{
    FilesystemMigrationAuthorityArtifact, FilesystemMigrationAuthorityError,
    StoreRootIdentityCoordinate,
};

impl fmt::Display for FilesystemMigrationAuthorityArtifact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Head => formatter.write_str("HEAD"),
            Self::Catalog { generation, digest } => {
                write!(formatter, "catalog {generation:?}/{digest:?}")
            }
        }
    }
}

impl fmt::Display for StoreRootIdentityCoordinate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Device => "device",
            Self::Mount => "mount",
            Self::File => "file",
        })
    }
}

impl fmt::Display for FilesystemMigrationAuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inventory { .. } => {
                formatter.write_str("filesystem migration inventory was refused")
            }
            Self::Namespace { .. } => {
                formatter.write_str("filesystem migration root namespace was refused")
            }
            Self::RootIdentity { .. } => {
                formatter.write_str("filesystem migration root identity could not be observed")
            }
            Self::RootIdentityChanged {
                coordinate,
                expected,
                observed,
            } => write!(
                formatter,
                "filesystem migration root {coordinate} changed: expected {expected}, observed \
                 {observed}"
            ),
            Self::Artifact { artifact, .. } => {
                write!(formatter, "filesystem migration could not read {artifact}")
            }
            Self::ArtifactChanged { artifact } => {
                write!(
                    formatter,
                    "filesystem migration {artifact} changed identity"
                )
            }
            Self::Head { .. } => formatter.write_str("filesystem migration HEAD was malformed"),
            Self::Catalog {
                generation, digest, ..
            } => write!(
                formatter,
                "filesystem migration catalog {generation:?}/{digest:?} was malformed"
            ),
            Self::CatalogGeneration { expected, observed } => write!(
                formatter,
                "filesystem migration catalog generation disagreed: expected {expected:?}, \
                 observed {observed:?}"
            ),
            Self::CatalogLength { expected, observed } => write!(
                formatter,
                "filesystem migration catalog length disagreed: expected {expected:?}, observed \
                 {observed:?}"
            ),
            Self::CatalogDigest { expected, observed } => write!(
                formatter,
                "filesystem migration catalog digest disagreed: expected {expected:?}, observed \
                 {observed:?}"
            ),
            Self::HeadChanged => {
                formatter.write_str("filesystem migration HEAD changed during observation")
            }
            Self::IntentChanged { expected, observed } => write!(
                formatter,
                "filesystem migration intent changed: expected {expected:?}, observed {observed:?}"
            ),
        }
    }
}

impl Error for FilesystemMigrationAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inventory { source } => Some(source),
            Self::Namespace { source } | Self::RootIdentity { source } => Some(source),
            Self::Artifact { source, .. } => Some(source),
            Self::Head { source } => Some(source),
            Self::Catalog { source, .. } => Some(source),
            Self::RootIdentityChanged { .. }
            | Self::ArtifactChanged { .. }
            | Self::CatalogGeneration { .. }
            | Self::CatalogLength { .. }
            | Self::CatalogDigest { .. }
            | Self::HeadChanged
            | Self::IntentChanged { .. } => None,
        }
    }
}
