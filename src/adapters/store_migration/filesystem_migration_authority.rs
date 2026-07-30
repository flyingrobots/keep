//! This module owns exact writer-locked filesystem migration authority.

use super::filesystem_inventory_file::{self, FilesystemInventoryFilePolicy};
use super::filesystem_migration_authority_error::{
    FilesystemMigrationAuthorityArtifact as Artifact, FilesystemMigrationAuthorityError as Error,
    StoreRootIdentityCoordinate as RootCoordinate,
};
use super::filesystem_migration_authority_validation::{
    artifact_error, require_root, verify_catalog,
};
use super::migration_catalog_coordinates::MigrationCatalogCoordinates;
use super::store_root_identity::StoreRootIdentities;
use super::{CanonicalStoreMigrationIntent, FilesystemStoreMigrationInventoryReader};
use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartPhase, ChecksummedCatalog, ChecksummedPublicationHead,
    FilesystemPlatformAdmission, SegmentReadPolicy, filesystem_initialization_namespace,
    filesystem_platform_profile, physical_pool_name,
};

const HEAD_NAME: &str = "HEAD";
const HEAD_LENGTH: u64 = 128;

/// Exclusive authority to observe and migrate one pinned version-1 filesystem root.
///
/// The authority retains the admitted writer lock and pinned root and immutable
/// pool capabilities for its entire lifetime. Its synchronous,
/// capability-relative filesystem I/O performs no protocol mutation and uses
/// neither a network nor an asynchronous runtime.
#[must_use]
pub struct FilesystemStoreMigrationAuthority {
    inventory: FilesystemStoreMigrationInventoryReader,
}

impl FilesystemStoreMigrationAuthority {
    /// Pins one admitted filesystem root for migration observation.
    ///
    /// This synchronous constructor opens pinned directory capabilities but
    /// materializes no artifact bodies and performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationAuthorityError`](super::FilesystemMigrationAuthorityError)
    /// when the root capability cannot be cloned or either immutable pool
    /// cannot be pinned without following links.
    pub fn open(
        admission: FilesystemPlatformAdmission,
        policy: SegmentReadPolicy,
    ) -> Result<Self, Error> {
        let inventory = FilesystemStoreMigrationInventoryReader::open(admission, policy)
            .map_err(|source| Error::Inventory { source })?;
        Ok(Self { inventory })
    }

    /// Observes one canonical intent from exact current version-1 authority.
    ///
    /// The synchronous call admits the exact published root namespace, physical
    /// root coordinate, fixed-width head, complete immutable-pool inventory,
    /// and head-selected catalog. Peak content allocation is bounded by one
    /// catalog and one segment in addition to the bounded semantic inventory.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationAuthorityError`](super::FilesystemMigrationAuthorityError)
    /// at the exact namespace, root, artifact, coordinate, or inventory refusal.
    pub fn observe_intent(&self) -> Result<CanonicalStoreMigrationIntent, Error> {
        self.verify_namespace()?;
        let roots = self.verify_root_identity()?;
        let head_bytes = self.read_head()?;
        let head = ChecksummedPublicationHead::decode(&head_bytes)
            .map_err(|source| Error::Head { source })?;
        let inventory_digest = self
            .inventory
            .read()
            .map_err(|source| Error::Inventory { source })?;
        let coordinates = self.read_catalog(head)?;
        if self.read_head()? != head_bytes {
            return Err(Error::HeadChanged);
        }
        self.verify_namespace()?;
        let _current_roots = self.verify_root_identity()?;
        Ok(CanonicalStoreMigrationIntent::from_coordinates(
            coordinates,
            inventory_digest,
            roots,
        ))
    }

    /// Re-observes and compares every coordinate in one canonical intent.
    ///
    /// This has the same synchronous I/O and bounded-allocation behavior as
    /// [`Self::observe_intent`] and performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns the exact observation refusal or
    /// [`FilesystemMigrationAuthorityError::IntentChanged`] with both intent
    /// digests when current authority no longer reproduces `expected`.
    pub fn verify_current(&self, expected: &CanonicalStoreMigrationIntent) -> Result<(), Error> {
        let observed = self.observe_intent()?;
        if &observed == expected {
            Ok(())
        } else {
            Err(Error::IntentChanged {
                expected: expected.digest(),
                observed: observed.digest(),
            })
        }
    }

    fn verify_namespace(&self) -> Result<(), Error> {
        filesystem_initialization_namespace::admit_published(self.inventory.root())
            .map_err(|source| Error::Namespace { source })
    }

    fn verify_root_identity(&self) -> Result<StoreRootIdentities, Error> {
        let expected = self.inventory.root_identity();
        let observed = filesystem_platform_profile::root_identity(self.inventory.root())
            .map_err(|source| Error::RootIdentity { source })?;
        require_root(RootCoordinate::Device, expected.device(), observed.device())?;
        require_root(RootCoordinate::Mount, expected.mount(), observed.mount())?;
        require_root(RootCoordinate::File, expected.file(), observed.file())?;
        Ok(StoreRootIdentities::from_filesystem(observed))
    }

    fn read_head(&self) -> Result<Vec<u8>, Error> {
        let policy = FilesystemInventoryFilePolicy::new(
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::OpenHead,
            CatalogRestartPhase::ReadHead,
            HEAD_LENGTH,
        );
        filesystem_inventory_file::read(self.inventory.root(), HEAD_NAME, policy)
            .map_err(|source| artifact_error(Artifact::Head, source))
    }

    fn read_catalog(
        &self,
        head: ChecksummedPublicationHead<'_>,
    ) -> Result<MigrationCatalogCoordinates, Error> {
        let artifact = Artifact::Catalog {
            generation: head.generation(),
            digest: head.catalog_digest(),
        };
        let name = physical_pool_name::catalog(head.generation(), head.catalog_digest());
        let policy = FilesystemInventoryFilePolicy::new(
            CatalogRestartArtifact::Catalog,
            CatalogRestartPhase::OpenCatalog,
            CatalogRestartPhase::ReadCatalog,
            head.catalog_length().get(),
        );
        let bytes = filesystem_inventory_file::read(self.inventory.catalogs(), &name, policy)
            .map_err(|source| artifact_error(artifact, source))?;
        let catalog = ChecksummedCatalog::decode(&bytes).map_err(|source| Error::Catalog {
            generation: head.generation(),
            digest: head.catalog_digest(),
            source: Box::new(source),
        })?;
        verify_catalog(head, catalog)
    }
}
