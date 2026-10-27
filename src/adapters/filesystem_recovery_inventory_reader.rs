//! This module owns capability-relative filesystem recovery inventory.

use std::io;
use std::path::Path;

#[cfg(any(test, feature = "repository-tasks"))]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_version_two_admission::admit_root;
use super::{
    FilesystemRecoveryStageError, RecoveryEntryName, RecoveryInventory, RecoveryInventoryError,
    RecoveryInventoryLimit, RecoveryInventoryOperation, RecoveryInventoryStorage,
    RecoveryNamespace, RecoveryStage, RecoveryStageCompletionPool, RecoveryStageEvidence,
    RecoveryStageNamespacePhase, RecoveryStageParent, filesystem_initialization_namespace,
    filesystem_platform_profile, filesystem_recovery_inventory_scan,
    filesystem_recovery_namespace::PinnedRecoveryDirectory, filesystem_recovery_stage,
    read_recovery_inventory,
};

const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

/// A pinned, read-only view of the four recovery inventory namespaces.
///
/// Opening performs no protocol mutation. The production constructor admits
/// the same Linux ext4 profile, namespace, and `FORMAT` marker as recovery
/// admission and pins all three child directories without following symbolic
/// links.
#[must_use]
pub struct FilesystemRecoveryInventoryReader {
    root: Dir,
    staging: PinnedRecoveryDirectory,
    segments: PinnedRecoveryDirectory,
    catalogs: PinnedRecoveryDirectory,
}

impl FilesystemRecoveryInventoryReader {
    /// Opens one committed store for read-only stage inventory.
    ///
    /// Admission is the recovery law: the production platform for every
    /// protocol directory, the exact namespace with an
    /// optional retained catalog `head.next`, and the canonical `FORMAT` marker.
    /// No writer lock is taken
    /// and no protocol state is mutated. Classify the resulting inventory with
    /// [`classify_version_two_recovery_names`](crate::classify_version_two_recovery_names).
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryInventoryError::Io`] for the store root when platform
    /// or version-two admission refuses; the source carries the exact
    /// [`FilesystemPlatformAdmissionError`](crate::FilesystemPlatformAdmissionError).
    /// A root without the `FORMAT` marker refuses.
    pub fn open_version_two(store_root: &Path) -> Result<Self, RecoveryInventoryError> {
        let root = filesystem_platform_profile::open_version_two(store_root).map_err(root_error)?;
        admit_root(
            &root,
            filesystem_initialization_namespace::admit_version_two_with_catalog_stage,
        )
        .map_err(|source| root_error(io::Error::new(io::ErrorKind::InvalidData, source)))?;
        Self::pin(root)
    }

    /// Opens a committed root without platform admission for repository tasks.
    ///
    /// Every namespace, record, and identity law of [`Self::open_version_two`]
    /// still applies.
    ///
    /// # Errors
    ///
    /// As [`Self::open_version_two`] after platform admission.
    #[cfg(any(test, feature = "repository-tasks"))]
    #[doc(hidden)]
    pub fn open_version_two_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<Self, RecoveryInventoryError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority()).map_err(root_error)?;
        admit_root(
            &root,
            filesystem_initialization_namespace::admit_version_two_with_catalog_stage,
        )
        .map_err(|source| root_error(io::Error::new(io::ErrorKind::InvalidData, source)))?;
        Self::pin(root)
    }

    /// Pins an already admitted, writer-locked version-two root.
    pub(super) fn from_version_two_root(root: Dir) -> Result<Self, RecoveryInventoryError> {
        filesystem_initialization_namespace::admit_version_two_with_catalog_stage(&root)
            .map_err(root_error)?;
        Self::pin(root)
    }

    fn pin(root: Dir) -> Result<Self, RecoveryInventoryError> {
        let staging =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Staging, STAGING_NAME)?;
        let segments =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Segments, SEGMENTS_NAME)?;
        let catalogs =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Catalogs, CATALOGS_NAME)?;
        Ok(Self {
            root,
            staging,
            segments,
            catalogs,
        })
    }

    /// Reads one bounded, deterministic inventory without mutating protocol
    /// state.
    ///
    /// The call is synchronous and may block on directory I/O. Peak allocation
    /// includes the final `limit`-bounded inventory plus one temporary
    /// `limit + 1`-bounded namespace table used to prove count drift.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryInventoryError`] on namespace replacement, storage
    /// refusal, entry-limit excess, count drift, or duplicate names.
    pub fn read(
        &mut self,
        limit: RecoveryInventoryLimit,
    ) -> Result<RecoveryInventory, RecoveryInventoryError> {
        self.verify_namespaces()?;
        let inventory = read_recovery_inventory(self, limit)?;
        self.verify_namespaces()?;
        Ok(inventory)
    }

    /// Produces bounded exact evidence for one fixed recovery stage.
    ///
    /// Opens relative to the pinned root or staging capability without
    /// following links, admits only a regular file, streams under the
    /// name-selected bound, then verifies the handle, entry, and namespaces.
    /// The synchronous call allocates no content-sized memory, may block on
    /// filesystem I/O, and performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryStageError`] on namespace drift, link or
    /// file refusal, oversized evidence, entry replacement, length drift, or
    /// underlying I/O failure.
    pub fn fingerprint_stage(
        &self,
        stage: RecoveryStage,
    ) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError> {
        self.verify_stage_namespaces(stage, RecoveryStageNamespacePhase::BeforeObservation)?;
        let evidence = filesystem_recovery_stage::fingerprint(self.stage_directory(stage), stage)?;
        self.verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)?;
        Ok(evidence)
    }

    #[cfg(test)]
    pub(super) fn fingerprint_stage_with<F>(
        &self,
        stage: RecoveryStage,
        after_open: F,
    ) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError>
    where
        F: FnOnce(),
    {
        self.verify_stage_namespaces(stage, RecoveryStageNamespacePhase::BeforeObservation)?;
        let evidence = filesystem_recovery_stage::fingerprint_with(
            self.stage_directory(stage),
            stage,
            after_open,
        )?;
        self.verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)?;
        Ok(evidence)
    }

    fn verify_namespaces(&self) -> Result<(), RecoveryInventoryError> {
        self.staging.verify(&self.root)?;
        self.segments.verify(&self.root)?;
        self.catalogs.verify(&self.root)
    }

    pub(super) fn verify_stage_namespaces(
        &self,
        stage: RecoveryStage,
        phase: RecoveryStageNamespacePhase,
    ) -> Result<(), FilesystemRecoveryStageError> {
        self.verify_namespaces()
            .map_err(|source| FilesystemRecoveryStageError::Namespace {
                stage,
                phase,
                source,
            })
    }

    const fn directory(&self, namespace: RecoveryNamespace) -> &Dir {
        match namespace {
            RecoveryNamespace::Root => &self.root,
            RecoveryNamespace::Staging => self.staging.directory(),
            RecoveryNamespace::Segments => self.segments.directory(),
            RecoveryNamespace::Catalogs => self.catalogs.directory(),
        }
    }

    pub(super) const fn parent_directory(&self, parent: RecoveryStageParent) -> &Dir {
        match parent {
            RecoveryStageParent::Staging => self.directory(RecoveryNamespace::Staging),
            RecoveryStageParent::Root => self.directory(RecoveryNamespace::Root),
        }
    }

    pub(super) const fn stage_directory(&self, stage: RecoveryStage) -> &Dir {
        self.parent_directory(stage.parent())
    }

    pub(super) const fn completion_pool_directory(
        &self,
        pool: RecoveryStageCompletionPool,
    ) -> &Dir {
        match pool {
            RecoveryStageCompletionPool::Segments => self.directory(RecoveryNamespace::Segments),
            RecoveryStageCompletionPool::Catalogs => self.directory(RecoveryNamespace::Catalogs),
        }
    }
}

impl RecoveryInventoryStorage for FilesystemRecoveryInventoryReader {
    fn count_entries(&mut self, namespace: RecoveryNamespace, remaining: u64) -> io::Result<u64> {
        filesystem_recovery_inventory_scan::count_entries(self.directory(namespace), remaining)
    }

    fn read_entry_names(
        &mut self,
        namespace: RecoveryNamespace,
        expected_count: u64,
    ) -> io::Result<Vec<RecoveryEntryName>> {
        filesystem_recovery_inventory_scan::read_entry_names(
            self.directory(namespace),
            expected_count,
        )
    }
}

const fn root_error(source: io::Error) -> RecoveryInventoryError {
    RecoveryInventoryError::io(
        RecoveryNamespace::Root,
        RecoveryInventoryOperation::OpenNamespace,
        source,
    )
}
