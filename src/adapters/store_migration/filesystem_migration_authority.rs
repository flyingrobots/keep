//! This module owns exact writer-locked filesystem migration authority.

use cap_fs_ext::DirExt;

use super::filesystem_inventory_file::{self, FilesystemInventoryFilePolicy};
use super::filesystem_migration_authority_error::{
    FilesystemMigrationAuthorityArtifact as Artifact, FilesystemMigrationAuthorityError as Error,
    StoreRootIdentityCoordinate as RootCoordinate,
};
use super::filesystem_migration_authority_validation::{
    artifact_error, require_root, verify_catalog,
};
use super::filesystem_migration_fixed_artifact::FilesystemMigrationFixedStage;
use super::migration_catalog_coordinates::MigrationCatalogCoordinates;
use super::store_root_identity::StoreRootIdentities;
use super::{CanonicalStoreMigrationIntent, FilesystemStoreMigrationInventoryReader};
use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartPhase, ChecksummedCatalog, ChecksummedPublicationHead,
    FilesystemPlatformAdmission, SegmentReadPolicy, filesystem_initialization_namespace,
    filesystem_platform_profile, physical_pool_name,
};

const HEAD_NAME: &str = "HEAD";
const STAGING_NAME: &str = "staging";
const HEAD_LENGTH: u64 = 128;

/// Exclusive authority to observe and migrate one pinned version-1 filesystem root.
///
/// The authority retains the admitted writer lock and pinned root and immutable
/// pool capabilities for its entire lifetime. Observation performs no protocol
/// mutation. When passed to [`crate::execute_store_migration`], its
/// [`crate::StoreMigrationStorage`] implementation executes only the fresh
/// forward protocol from an exactly admitted version-1 root. It retains opened
/// fixed-record handles through final verification, performs synchronous
/// capability-relative I/O, and uses neither a network nor an asynchronous
/// runtime. Reopening a partial migration prefix remains a separate recovery
/// boundary.
#[must_use]
pub struct FilesystemStoreMigrationAuthority {
    inventory: FilesystemStoreMigrationInventoryReader,
    pub(super) fixed_stage: Option<FilesystemMigrationFixedStage>,
    pub(super) published_intent: Option<FilesystemMigrationFixedStage>,
    pub(super) published_marker: Option<FilesystemMigrationFixedStage>,
    pub(super) published_receipt: Option<FilesystemMigrationFixedStage>,
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
        Ok(Self {
            inventory,
            fixed_stage: None,
            published_intent: None,
            published_marker: None,
            published_receipt: None,
        })
    }

    /// Pins a root for migration without platform admission for repository tasks.
    ///
    /// Repository tools such as the crash matrix run on hosts outside the
    /// admitted Linux profile; namespace, head, catalog, inventory, and record
    /// laws still apply in full. Production callers use [`Self::open`].
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationAuthorityError`](super::FilesystemMigrationAuthorityError)
    /// when the root identity cannot be read or the pools cannot be pinned.
    #[cfg(feature = "repository-tasks")]
    pub fn open_unchecked_for_repository_tasks(
        lock: crate::adapters::FilesystemWriterLock,
        policy: SegmentReadPolicy,
    ) -> Result<Self, Error> {
        let admission = FilesystemPlatformAdmission::unchecked_for_repository_tasks(lock)
            .map_err(|source| Error::RootIdentity { source })?;
        Self::open(admission, policy)
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
    /// [`Error::IntentChanged`] with both intent
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

    /// Admits the exact published version-one root and requires `staging` to
    /// hold nothing.
    ///
    /// A retained `current.seg` or `current.cat` is version-one recovery
    /// evidence. Migration never inventories `staging`, and once the version-two
    /// markers exist the version-one recovery constructors refuse the root, so
    /// migrating over a retained stage would strand a recoverable crash state.
    /// Recovery must complete before the intent is observed.
    fn verify_namespace(&self) -> Result<(), Error> {
        let root = self.inventory.root();
        filesystem_initialization_namespace::admit_published(root)
            .map_err(|source| Error::Namespace { source })?;
        let staging = root
            .open_dir_nofollow(STAGING_NAME)
            .map_err(|source| Error::Namespace { source })?;
        let mut entries = staging
            .entries()
            .map_err(|source| Error::Namespace { source })?;
        match entries.next().transpose() {
            Ok(None) => Ok(()),
            Ok(Some(_entry)) => Err(Error::Namespace {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "version-one staging holds a retained stage; recover it before migration",
                ),
            }),
            Err(source) => Err(Error::Namespace { source }),
        }
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

    pub(super) const fn root(&self) -> &cap_std::fs::Dir {
        self.inventory.root()
    }
}
