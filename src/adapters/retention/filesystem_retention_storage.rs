//! This module owns forward filesystem retention publication execution.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_attempt::{self as attempt, PublicationAttempt};
use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_catalog;
use super::filesystem_retention_current::{self, ObservedDisposition};
use super::filesystem_retention_namespace;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::FilesystemRetentionStage;
use super::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    FilesystemRetentionRecoveryError, RetentionCurrentStateRefusal, RetentionNamespaceAdmission,
    RetentionPublicationPreparation, RetentionPublicationStorage, RetentionRecoveryOutcome,
    RetentionTransitionDisposition,
};
use crate::RetentionGenerationExpectation;
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record::EntryIdentity;

impl RetentionPublicationStorage for FilesystemRetentionPublicationAuthority {
    fn verify_current(
        &mut self,
        preparation: &RetentionPublicationPreparation<'_>,
    ) -> io::Result<RetentionTransitionDisposition> {
        self.attempt = None;
        require_pinned_directories(&self.root, &self.retention, &self.roots, &self.manifests)?;
        let recovery = self.recover().map_err(|error| match error {
            FilesystemRetentionRecoveryError::Observe { source } => source,
            FilesystemRetentionRecoveryError::Plan { source } => {
                RetentionCurrentStateRefusal::RecoveryRefused { source }.into_io()
            }
            FilesystemRetentionRecoveryError::Execute { source } => {
                RetentionCurrentStateRefusal::RecoveryStepRefused { source }.into_io()
            }
        })?;
        if matches!(
            recovery.outcome(),
            RetentionRecoveryOutcome::Protected { .. }
        ) {
            return Err(RetentionCurrentStateRefusal::RetainedStage.into_io());
        }
        require_no_retained_stage(&self.retention)?;
        let census =
            filesystem_retention_namespace::admit(&self.retention, &self.roots, &self.manifests)?;
        let current = filesystem_retention_current::observe(&self.retention, &self.manifests)?;
        if current.is_none() && !census.is_empty() {
            return Err(RetentionCurrentStateRefusal::HeadAbsentWithArtifacts.into_io());
        }
        let observed = filesystem_retention_current::disposition(preparation, current.as_ref())?;
        filesystem_retention_catalog::require_current_catalog(&self.root, preparation)?;
        match observed {
            ObservedDisposition::Publish => {
                filesystem_retention_namespace::admit_expectation(
                    &self.roots,
                    preparation.candidate(),
                    preparation.expected(),
                )?;
                filesystem_retention_namespace::admit_capacity(
                    census,
                    &self.roots,
                    preparation.candidate(),
                )?;
                if let (RetentionGenerationExpectation::Current(_), Some(current)) =
                    (preparation.expected(), current.as_ref())
                {
                    filesystem_retention_current::verify_predecessor(
                        &self.roots,
                        current,
                        preparation.candidate(),
                    )?;
                }
                self.attempt = Some(PublicationAttempt::new(
                    preparation.expected(),
                    preparation.liveness_generation(),
                ));
            }
            ObservedDisposition::Committed(current) => {
                filesystem_retention_current::verify_committed(
                    &self.roots,
                    current,
                    preparation.candidate(),
                )?;
            }
        }
        Ok(observed.transition())
    }

    fn write_root_stage(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let stage = FilesystemRetentionStage::create(
            &self.retention,
            pool_name::ROOT_STAGE,
            root.encoded(),
        )?;
        attempt.retain_root_stage(stage);
        Ok(())
    }

    fn synchronize_root_stage(&mut self) -> io::Result<()> {
        attempt::require(self.attempt.as_ref())?
            .root_stage()?
            .synchronize(&self.retention)?;
        synchronize_directory(&self.retention)
    }

    fn admit_root_namespace(
        &mut self,
        root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let name = pool_name::namespace(root.root().namespace().digest());
        let admission = match attempt.expected() {
            RetentionGenerationExpectation::Absent => match self.roots.create_dir(&name) {
                Ok(()) => RetentionNamespaceAdmission::Created,
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                    return Err(
                        RetentionCurrentStateRefusal::NamespaceExpectationViolated.into_io()
                    );
                }
                Err(source) => return Err(source),
            },
            RetentionGenerationExpectation::Current(_) => RetentionNamespaceAdmission::Existing,
        };
        let namespace = match self.roots.open_dir_nofollow(&name) {
            Ok(namespace) => namespace,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Err(RetentionCurrentStateRefusal::NamespaceExpectationViolated.into_io());
            }
            Err(source) => return Err(source),
        };
        attempt.retain_namespace(name, namespace);
        Ok(admission)
    }

    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()> {
        synchronize_directory(&self.roots)
    }

    fn link_root(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let namespace = pool_name::namespace(root.root().namespace().digest());
        let name = pool_name::root(root.root().generation(), root.digest());
        attempt.root_stage()?.link(
            &self.retention,
            attempt.require_namespace(&namespace)?,
            &name,
        )?;
        attempt.retain_root_name(name);
        Ok(())
    }

    fn synchronize_root_namespace(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        let namespace = pool_name::namespace(root.root().namespace().digest());
        synchronize_directory(
            attempt::require(self.attempt.as_ref())?.require_namespace(&namespace)?,
        )
    }

    fn write_manifest_stage(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let stage = FilesystemRetentionStage::create(
            &self.retention,
            pool_name::MANIFEST_STAGE,
            manifest.encoded(),
        )?;
        attempt.retain_manifest_stage(stage);
        Ok(())
    }

    fn synchronize_manifest_stage(&mut self) -> io::Result<()> {
        attempt::require(self.attempt.as_ref())?
            .manifest_stage()?
            .synchronize(&self.retention)?;
        synchronize_directory(&self.retention)
    }

    fn link_manifest(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let name = attempt.manifest_name(manifest);
        attempt
            .manifest_stage()?
            .link(&self.retention, &self.manifests, &name)?;
        attempt.retain_manifest_name(name);
        Ok(())
    }

    fn synchronize_manifest_pool(&mut self) -> io::Result<()> {
        synchronize_directory(&self.manifests)
    }

    fn write_head_stage(&mut self, head: &CanonicalRetentionHead) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let stage = FilesystemRetentionStage::create(
            &self.retention,
            pool_name::HEAD_STAGE,
            head.encoded(),
        )?;
        attempt.retain_head_stage(stage);
        Ok(())
    }

    fn synchronize_head_stage(&mut self) -> io::Result<()> {
        attempt::require(self.attempt.as_ref())?
            .head_stage()?
            .synchronize(&self.retention)?;
        synchronize_directory(&self.retention)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        attempt::require_mut(&mut self.attempt)?
            .take_head_stage()?
            .replace(&self.retention, pool_name::HEAD)
    }

    fn synchronize_retention_namespace(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let stage = attempt.take_root_stage()?;
        stage.remove(
            &self.retention,
            attempt.namespace()?,
            attempt.retained_root_name()?,
        )
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        let attempt = attempt::require_mut(&mut self.attempt)?;
        let stage = attempt.take_manifest_stage()?;
        stage.remove(
            &self.retention,
            &self.manifests,
            attempt.retained_manifest_name()?,
        )
    }

    fn synchronize_cleanup(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)?;
        self.attempt = None;
        Ok(())
    }
}

/// Requires the protocol names to still resolve to the directories admission pinned.
///
/// The authority operates only on the pinned capabilities, but a `retention`,
/// `roots`, or `manifests` entry renamed and replaced after admission means the
/// store's namespace no longer describes the admitted state; publication
/// refuses instead of writing into a directory no reader would find.
fn require_pinned_directories(
    root: &Dir,
    retention: &Dir,
    roots: &Dir,
    manifests: &Dir,
) -> io::Result<()> {
    for (parent, name, pinned) in [
        (root, pool_name::RETENTION, retention),
        (retention, pool_name::ROOTS, roots),
        (retention, pool_name::MANIFESTS, manifests),
    ] {
        let current = match parent.open_dir_nofollow(name) {
            Ok(current) => current,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Err(RetentionCurrentStateRefusal::ProtocolDirectoryReplaced.into_io());
            }
            Err(source) => return Err(source),
        };
        if EntryIdentity::of_directory(&current)? != EntryIdentity::of_directory(pinned)? {
            return Err(RetentionCurrentStateRefusal::ProtocolDirectoryReplaced.into_io());
        }
    }
    Ok(())
}

fn require_no_retained_stage(retention: &Dir) -> io::Result<()> {
    for stage in [
        pool_name::ROOT_STAGE,
        pool_name::MANIFEST_STAGE,
        pool_name::HEAD_STAGE,
    ] {
        match retention.symlink_metadata(stage) {
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(RetentionCurrentStateRefusal::RetainedStage.into_io());
            }
            Err(source) => return Err(source),
        }
    }
    Ok(())
}
