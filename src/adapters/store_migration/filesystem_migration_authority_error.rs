//! This boundary module owns filesystem migration-authority failures.

use std::io;

use super::{FilesystemMigrationInventoryError, StoreMigrationIntentDigest};
use crate::adapters::{CatalogDecodeError, CatalogRestartError, PublicationHeadDecodeError};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Published version-1 artifact observed while establishing migration authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemMigrationAuthorityArtifact {
    /// The mutable published `HEAD`.
    Head,
    /// The immutable catalog selected by `HEAD`.
    Catalog {
        /// Catalog generation named by `HEAD`.
        generation: CatalogGeneration,
        /// Catalog digest named by `HEAD`.
        digest: CatalogDigest,
    },
}

/// Physical store-root coordinate compared during migration admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreRootIdentityCoordinate {
    /// Platform device coordinate.
    Device,
    /// Platform mount coordinate.
    Mount,
    /// Platform file coordinate.
    File,
}

/// Failure to observe or revalidate exact filesystem migration authority.
#[derive(Debug)]
pub enum FilesystemMigrationAuthorityError {
    /// Complete immutable-pool inventory could not be admitted.
    Inventory {
        /// Preserved inventory refusal.
        source: FilesystemMigrationInventoryError,
    },
    /// The exact published version-1 root namespace could not be admitted.
    Namespace {
        /// Preserved capability-relative filesystem source.
        source: io::Error,
    },
    /// The physical root identity could not be observed.
    RootIdentity {
        /// Preserved platform source.
        source: io::Error,
    },
    /// One physical root coordinate changed under retained authority.
    RootIdentityChanged {
        /// Coordinate that changed.
        coordinate: StoreRootIdentityCoordinate,
        /// Coordinate retained by platform admission.
        expected: u64,
        /// Coordinate observed immediately before migration.
        observed: u64,
    },
    /// One selected artifact could not be read completely.
    Artifact {
        /// Exact artifact being observed.
        artifact: FilesystemMigrationAuthorityArtifact,
        /// Preserved bounded-read refusal.
        source: Box<CatalogRestartError>,
    },
    /// One selected artifact changed physical identity while being read.
    ArtifactChanged {
        /// Exact artifact that changed.
        artifact: FilesystemMigrationAuthorityArtifact,
    },
    /// The published head bytes were malformed.
    Head {
        /// Preserved head-decoding refusal.
        source: PublicationHeadDecodeError,
    },
    /// The selected catalog bytes were malformed.
    Catalog {
        /// Catalog generation selected by the head.
        generation: CatalogGeneration,
        /// Catalog digest selected by the head.
        digest: CatalogDigest,
        /// Preserved catalog-decoding refusal.
        source: Box<CatalogDecodeError>,
    },
    /// Head and selected catalog generation coordinates disagreed.
    CatalogGeneration {
        /// Generation required by the head.
        expected: CatalogGeneration,
        /// Generation observed in the catalog.
        observed: CatalogGeneration,
    },
    /// Head and selected catalog length coordinates disagreed.
    CatalogLength {
        /// Length required by the head.
        expected: CatalogLength,
        /// Length observed in the catalog.
        observed: CatalogLength,
    },
    /// Head and selected catalog digest coordinates disagreed.
    CatalogDigest {
        /// Digest required by the head.
        expected: CatalogDigest,
        /// Digest observed in the catalog.
        observed: CatalogDigest,
    },
    /// The mutable head changed during authority observation.
    HeadChanged,
    /// Re-observation did not reproduce the supplied canonical intent.
    IntentChanged {
        /// Intent authorized by the caller.
        expected: StoreMigrationIntentDigest,
        /// Intent derived from current filesystem authority.
        observed: StoreMigrationIntentDigest,
    },
}
