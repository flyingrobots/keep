//! This module owns the migration authority's repository crash-task hooks.
//!
//! Repository process-death tasks drive the production migration protocol
//! from a store they built without platform admission, stop it at an exact
//! byte or directory prefix, and reopen it for recovery. Nothing here is a
//! separate protocol: each hook runs one production step short, or opens the
//! same authority without the Linux platform check.

use std::io;
use std::path::Path;

use cap_std::fs::Dir;

use super::filesystem_migration_authority_error::FilesystemMigrationAuthorityError as Error;
use super::filesystem_migration_fixed_artifact::{
    FilesystemMigrationFixedArtifact as FixedArtifact, FilesystemMigrationFixedStage,
};
use super::{
    FilesystemStoreMigrationAuthority, StoreMigrationFixedStage, filesystem_migration_namespace,
};
use crate::adapters::SegmentReadPolicy;

impl FilesystemStoreMigrationAuthority {
    /// Reacquires recovery authority without platform admission, for
    /// repository process-death tasks; otherwise exactly
    /// [`Self::reopen_for_recovery`].
    ///
    /// # Errors
    ///
    /// Returns the same writer-lock, namespace, and pool failures as
    /// [`Self::reopen_for_recovery`].
    #[doc(hidden)]
    pub fn reopen_for_recovery_unchecked_for_repository_tasks(
        store_root: &Path,
        policy: SegmentReadPolicy,
    ) -> Result<Self, Error> {
        let root = Dir::open_ambient_dir(store_root, cap_std::ambient_authority())
            .map_err(|source| Error::Platform { source })?;
        Self::recover_root(root, policy)
    }

    /// Creates `stage` and writes a strict prefix of `record`, leaving an
    /// incomplete pre-effect stage exactly as process death during the write
    /// would.
    ///
    /// # Errors
    ///
    /// Returns the exact length, prefix-bound, creation, or write failure.
    #[doc(hidden)]
    pub fn write_fixed_stage_prefix_for_repository_tasks(
        &mut self,
        stage: StoreMigrationFixedStage,
        record: &[u8],
        prefix: usize,
    ) -> io::Result<()> {
        let artifact = match stage {
            StoreMigrationFixedStage::Intent => FixedArtifact::Intent,
            StoreMigrationFixedStage::Marker => FixedArtifact::Marker,
            StoreMigrationFixedStage::Receipt => FixedArtifact::Receipt,
        };
        FilesystemMigrationFixedStage::create_prefix(self.root(), artifact, record, prefix)
    }

    /// Admits only the first `count` namespace-prefix directories in protocol
    /// order, each with its parent synchronized, and stops before the final
    /// prefix verification.
    ///
    /// # Errors
    ///
    /// Returns the exact preflight or admission failure, or refuses a count
    /// beyond the six-directory prefix.
    #[doc(hidden)]
    pub fn admit_namespace_prefix_for_repository_tasks(&mut self, count: usize) -> io::Result<()> {
        filesystem_migration_namespace::admit_namespace_prefix_partially(self.root(), count)
    }
}
