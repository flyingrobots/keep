//! This module binds filesystem migration authority to the recovery port.

use std::io;
use std::path::Path;

use cap_std::fs::Dir;

use super::filesystem_migration_authority::MigrationNamespacePolicy;
use super::filesystem_migration_authority_error::FilesystemMigrationAuthorityError as Error;
use super::filesystem_migration_fixed_artifact::{
    FilesystemMigrationFixedArtifact as FixedArtifact, FilesystemMigrationFixedStage,
};
use super::migration_resumption::MigrationRecords;
use super::{
    CanonicalStoreMigrationIntent, FilesystemStoreMigrationAuthority,
    FilesystemStoreMigrationInventoryReader, StoreMigrationFixedStage,
    StoreMigrationRecoveryStorage, StoreMigrationResidue, filesystem_migration_residue,
};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::{
    FilesystemWriterLock, SegmentReadPolicy, filesystem_catalog_artifact,
    filesystem_initialization_namespace, filesystem_platform_profile,
};

impl FilesystemStoreMigrationAuthority {
    /// Reacquires writer authority over a root that may carry migration residue.
    ///
    /// The call admits the production platform, acquires the existing writer
    /// lock, and admits the published version-1 namespace plus any subset of
    /// migration records, stages, the reader fence, and protocol directories.
    /// It never produces a version-1 platform admission, so the version-1
    /// publisher can never run against a partly migrated root. The returned
    /// authority observes with the migrating namespace policy and implements
    /// [`StoreMigrationRecoveryStorage`]. It mutates nothing.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemMigrationAuthorityError`](Error) at the exact
    /// platform, writer-lock, namespace, or pool refusal.
    pub fn reopen_for_recovery(
        store_root: &Path,
        policy: SegmentReadPolicy,
    ) -> Result<Self, Error> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| Error::Platform { source })?;
        Self::recover_root(root, policy)
    }

    #[cfg(test)]
    pub(super) fn reopen_for_recovery_unchecked_for_tests(
        store_root: &Path,
        policy: SegmentReadPolicy,
    ) -> Result<Self, Error> {
        let root = Dir::open_ambient_dir(store_root, cap_std::ambient_authority())
            .map_err(|source| Error::Platform { source })?;
        Self::recover_root(root, policy)
    }

    fn recover_root(root: Dir, policy: SegmentReadPolicy) -> Result<Self, Error> {
        let lock = FilesystemWriterLock::try_acquire_in(root)
            .map_err(|source| Error::WriterLock { source })?;
        let directory = lock
            .clone_directory()
            .map_err(|source| Error::Namespace { source })?;
        filesystem_initialization_namespace::admit_migrating(&directory)
            .map_err(|source| Error::Namespace { source })?;
        let root_identity = filesystem_platform_profile::root_identity(&directory)
            .map_err(|source| Error::RootIdentity { source })?;
        let inventory =
            FilesystemStoreMigrationInventoryReader::open_locked(lock, root_identity, policy)
                .map_err(|source| Error::Inventory { source })?;
        Ok(Self::with_policy(
            inventory,
            MigrationNamespacePolicy::Migrating,
        ))
    }
}

impl StoreMigrationRecoveryStorage for FilesystemStoreMigrationAuthority {
    fn observe_residue(&mut self) -> io::Result<StoreMigrationResidue> {
        filesystem_migration_residue::observe(self.root())
    }

    fn adopt_residue(
        &mut self,
        residue: &StoreMigrationResidue,
        intent: &CanonicalStoreMigrationIntent,
    ) -> io::Result<()> {
        let records = MigrationRecords::for_intent(intent);
        adopt_artifact(
            self,
            FixedArtifact::Intent,
            residue.intent_stage.as_deref(),
            residue.intent.as_deref(),
            intent.encoded(),
        )?;
        adopt_artifact(
            self,
            FixedArtifact::Marker,
            residue.marker_stage.as_deref(),
            residue.marker.as_deref(),
            records.marker.encoded(),
        )?;
        adopt_artifact(
            self,
            FixedArtifact::Receipt,
            residue.receipt_stage.as_deref(),
            residue.receipt.as_deref(),
            records.receipt.encoded(),
        )
    }

    fn discard_stage(&mut self, stage: StoreMigrationFixedStage) -> io::Result<()> {
        let artifact = match stage {
            StoreMigrationFixedStage::Intent => FixedArtifact::Intent,
            StoreMigrationFixedStage::Marker => FixedArtifact::Marker,
            StoreMigrationFixedStage::Receipt => FixedArtifact::Receipt,
        };
        let root = self.root();
        exact_record::require_absent(root, artifact.canonical_name()).map_err(refusal)?;
        let metadata = root.symlink_metadata(artifact.stage_name())?;
        let complete = u64::try_from(artifact.encoded_length())
            .map_err(|_| invalid("migration stage length exceeded u64"))?;
        if !metadata.is_file() || metadata.len() >= complete {
            return Err(invalid(
                "only an incomplete regular pre-effect stage may be discarded",
            ));
        }
        root.remove_file(artifact.stage_name())?;
        exact_record::require_absent(root, artifact.stage_name()).map_err(refusal)?;
        filesystem_catalog_artifact::synchronize_directory(root)
    }
}

/// Reopens the handles one artifact's residue implies: an exact stage becomes
/// the active stage (shared with its canonical target when both exist), a
/// canonical record alone becomes the published handle, and an incomplete
/// stage is left for the planner's discard.
fn adopt_artifact(
    authority: &mut FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
    stage: Option<&[u8]>,
    canonical: Option<&[u8]>,
    expected: &[u8],
) -> io::Result<()> {
    let root = authority.root();
    if stage.is_some_and(|bytes| bytes == expected) {
        if authority.fixed_stage.is_some() {
            return Err(invalid("migration residue holds more than one exact stage"));
        }
        let reopened = FilesystemMigrationFixedStage::reopen_stage(root, artifact, expected)?;
        authority.fixed_stage = Some(reopened);
        return Ok(());
    }
    if canonical.is_some() {
        let reopened = FilesystemMigrationFixedStage::reopen_canonical(root, artifact, expected)?;
        match artifact {
            FixedArtifact::Intent => authority.published_intent = Some(reopened),
            FixedArtifact::Marker => authority.published_marker = Some(reopened),
            FixedArtifact::Receipt => authority.published_receipt = Some(reopened),
        }
    }
    Ok(())
}

fn refusal(error: ExactRecordError) -> io::Error {
    match error {
        ExactRecordError::Io(source) => source,
        ExactRecordError::Refused(refusal) => {
            io::Error::new(io::ErrorKind::InvalidData, refusal.to_string())
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
