//! This module binds filesystem migration authority to the storage port.

use std::io;

use super::FilesystemMigrationRefusal as Refusal;
use super::filesystem_migration_fixed_artifact::{
    FilesystemMigrationFixedArtifact as FixedArtifact, FilesystemMigrationFixedStage,
};
use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    FilesystemStoreMigrationAuthority, StoreMigrationStorage, filesystem_migration_namespace,
};
use crate::adapters::filesystem_catalog_artifact;

impl StoreMigrationStorage for FilesystemStoreMigrationAuthority {
    fn verify_current(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        verify_authority(self, intent)
    }

    fn write_intent_stage(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        write_stage(self, FixedArtifact::Intent, intent.encoded())
    }

    fn synchronize_intent_stage(&mut self) -> io::Result<()> {
        synchronize_stage(self, FixedArtifact::Intent)
    }

    fn link_intent(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()> {
        link_stage(self, FixedArtifact::Intent, intent.encoded())
    }

    fn synchronize_root_after_intent(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        active_stage(self, FixedArtifact::Intent)?.verify_linked(self.root())
    }

    fn remove_intent_stage(&mut self) -> io::Result<()> {
        remove_stage(self, FixedArtifact::Intent)
    }

    fn synchronize_root_after_intent_cleanup(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_published(self, FixedArtifact::Intent)?;
        filesystem_migration_namespace::verify_intent_root(self.root())
    }

    fn admit_reader_fence(&mut self) -> io::Result<()> {
        verify_published(self, FixedArtifact::Intent)?;
        filesystem_migration_namespace::admit_reader_fence(self.root())?;
        verify_published(self, FixedArtifact::Intent)
    }

    fn admit_namespace_prefix(&mut self) -> io::Result<()> {
        verify_published(self, FixedArtifact::Intent)?;
        filesystem_migration_namespace::admit_namespace_prefix(self.root())?;
        verify_published(self, FixedArtifact::Intent)
    }

    fn synchronize_root_after_namespace(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_published(self, FixedArtifact::Intent)?;
        filesystem_migration_namespace::verify_namespace_prefix(self.root())
    }

    fn write_marker_stage(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        verify_namespace_view(self)?;
        write_stage(self, FixedArtifact::Marker, marker.encoded())
    }

    fn synchronize_marker_stage(&mut self) -> io::Result<()> {
        synchronize_stage(self, FixedArtifact::Marker)
    }

    fn link_marker(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()> {
        link_stage(self, FixedArtifact::Marker, marker.encoded())
    }

    fn synchronize_root_after_marker(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_published(self, FixedArtifact::Intent)?;
        filesystem_migration_namespace::verify_namespace_contents(self.root())?;
        active_stage(self, FixedArtifact::Marker)?.verify_linked(self.root())
    }

    fn remove_marker_stage(&mut self) -> io::Result<()> {
        remove_stage(self, FixedArtifact::Marker)
    }

    fn synchronize_root_after_marker_cleanup(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_marker_view(self)
    }

    fn write_receipt_stage(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        verify_marker_view(self)?;
        write_stage(self, FixedArtifact::Receipt, receipt.encoded())
    }

    fn synchronize_receipt_stage(&mut self) -> io::Result<()> {
        synchronize_stage(self, FixedArtifact::Receipt)
    }

    fn link_receipt(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()> {
        link_stage(self, FixedArtifact::Receipt, receipt.encoded())
    }

    fn synchronize_root_after_receipt(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_published(self, FixedArtifact::Intent)?;
        verify_published(self, FixedArtifact::Marker)?;
        filesystem_migration_namespace::verify_marker_contents(self.root())?;
        active_stage(self, FixedArtifact::Receipt)?.verify_linked(self.root())
    }

    fn remove_receipt_stage(&mut self) -> io::Result<()> {
        remove_stage(self, FixedArtifact::Receipt)
    }

    fn synchronize_root_after_receipt_cleanup(&mut self) -> io::Result<()> {
        synchronize_root(self)?;
        verify_receipt_view(self)
    }
}

fn verify_authority(
    authority: &FilesystemStoreMigrationAuthority,
    intent: &CanonicalStoreMigrationIntent,
) -> io::Result<()> {
    authority.verify_current(intent).map_err(io::Error::other)
}

fn write_stage(
    authority: &mut FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
    expected: &[u8],
) -> io::Result<()> {
    if let Some(active) = authority.fixed_stage.as_ref() {
        return Err(stage_state(Refusal::StageAlreadyActive {
            active: active.artifact().stage(),
        }));
    }
    let stage = FilesystemMigrationFixedStage::create(authority.root(), artifact, expected)?;
    authority.fixed_stage = Some(stage);
    Ok(())
}

fn synchronize_stage(
    authority: &FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
) -> io::Result<()> {
    active_stage(authority, artifact)?.synchronize(authority.root())
}

fn link_stage(
    authority: &FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
    expected: &[u8],
) -> io::Result<()> {
    active_stage(authority, artifact)?.link(authority.root(), artifact, expected)
}

fn remove_stage(
    authority: &mut FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
) -> io::Result<()> {
    if published_stage(authority, artifact).is_some() {
        return Err(stage_state(Refusal::RecordAlreadyPublished {
            stage: artifact.stage(),
        }));
    }
    let stage = authority
        .fixed_stage
        .take()
        .ok_or_else(|| inactive(artifact))?;
    let observed = stage.artifact();
    if observed != artifact {
        authority.fixed_stage = Some(stage);
        return Err(different(artifact, observed));
    }
    let published = stage.remove(authority.root())?;
    match artifact {
        FixedArtifact::Intent => authority.published_intent = Some(published),
        FixedArtifact::Marker => authority.published_marker = Some(published),
        FixedArtifact::Receipt => authority.published_receipt = Some(published),
    }
    Ok(())
}

fn active_stage(
    authority: &FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
) -> io::Result<&FilesystemMigrationFixedStage> {
    let stage = authority
        .fixed_stage
        .as_ref()
        .ok_or_else(|| inactive(artifact))?;
    if stage.artifact() == artifact {
        Ok(stage)
    } else {
        Err(different(artifact, stage.artifact()))
    }
}

fn verify_published(
    authority: &FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
) -> io::Result<()> {
    published_stage(authority, artifact)
        .ok_or_else(|| {
            stage_state(Refusal::RecordNotPublished {
                stage: artifact.stage(),
            })
        })?
        .verify_canonical(authority.root())
}

const fn published_stage(
    authority: &FilesystemStoreMigrationAuthority,
    artifact: FixedArtifact,
) -> Option<&FilesystemMigrationFixedStage> {
    match artifact {
        FixedArtifact::Intent => authority.published_intent.as_ref(),
        FixedArtifact::Marker => authority.published_marker.as_ref(),
        FixedArtifact::Receipt => authority.published_receipt.as_ref(),
    }
}

fn verify_namespace_view(authority: &FilesystemStoreMigrationAuthority) -> io::Result<()> {
    verify_published(authority, FixedArtifact::Intent)?;
    filesystem_migration_namespace::verify_namespace_prefix(authority.root())
}

fn verify_marker_view(authority: &FilesystemStoreMigrationAuthority) -> io::Result<()> {
    verify_published(authority, FixedArtifact::Intent)?;
    verify_published(authority, FixedArtifact::Marker)?;
    filesystem_migration_namespace::verify_marker_view(authority.root())
}

fn verify_receipt_view(authority: &FilesystemStoreMigrationAuthority) -> io::Result<()> {
    verify_published(authority, FixedArtifact::Intent)?;
    verify_published(authority, FixedArtifact::Marker)?;
    verify_published(authority, FixedArtifact::Receipt)?;
    filesystem_migration_namespace::verify_receipt_view(authority.root())
}

fn synchronize_root(authority: &FilesystemStoreMigrationAuthority) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(authority.root())
}

fn inactive(expected: FixedArtifact) -> io::Error {
    stage_state(Refusal::StageInactive {
        expected: expected.stage(),
    })
}

fn different(expected: FixedArtifact, observed: FixedArtifact) -> io::Error {
    stage_state(Refusal::DifferentStageActive {
        expected: expected.stage(),
        observed: observed.stage(),
    })
}

fn stage_state(refusal: Refusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}
