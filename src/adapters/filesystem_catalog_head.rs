//! This module owns filesystem publication-head transitions.

use std::io::{self, Write};

use super::filesystem_catalog_stages::FilesystemCatalogStages;
use super::{
    CanonicalPublicationHead, CatalogRestartArtifact, CatalogSnapshot,
    FilesystemCatalogPublicationError, catalog_restart_loader, filesystem_catalog_artifact,
    filesystem_catalog_publisher,
};

pub(super) fn create_stage(stages: &mut FilesystemCatalogStages) -> io::Result<()> {
    if stages.head_stage.is_some() {
        return Err(stage_state());
    }
    stages.head_stage = Some(filesystem_catalog_artifact::create_exclusive(
        &stages.root,
        filesystem_catalog_publisher::NEXT_HEAD,
    )?);
    Ok(())
}

pub(super) fn write(
    stages: &mut FilesystemCatalogStages,
    head: &CanonicalPublicationHead,
) -> io::Result<()> {
    write_bytes(stages, head.encoded())
}

#[cfg(feature = "repository-tasks")]
pub(super) fn write_prefix(
    stages: &mut FilesystemCatalogStages,
    head: &CanonicalPublicationHead,
    prefix: usize,
) -> io::Result<()> {
    let bytes = head
        .encoded()
        .get(..prefix)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "head prefix exceeds bytes"))?;
    write_bytes(stages, bytes)
}

fn write_bytes(stages: &mut FilesystemCatalogStages, bytes: &[u8]) -> io::Result<()> {
    stage_mut(stages)?.write_all(bytes)
}

pub(super) fn flush(stages: &mut FilesystemCatalogStages) -> io::Result<()> {
    stage_mut(stages)?.flush()
}

pub(super) fn synchronize(stages: &mut FilesystemCatalogStages) -> io::Result<()> {
    stage_mut(stages)?.sync_all()
}

pub(super) fn verify_view(
    stages: &mut FilesystemCatalogStages,
    head: &CanonicalPublicationHead,
    expected: &CatalogSnapshot<'_, '_, '_>,
) -> io::Result<()> {
    let stage = stages.head_stage.take().ok_or_else(stage_state)?;
    drop(stage);
    let observed = catalog_restart_loader::load_from_directory(
        &stages.root,
        filesystem_catalog_publisher::NEXT_HEAD,
        stages.policy,
    )
    .map_err(filesystem_catalog_artifact::invalid_data)?;
    if observed.head_bytes() != head.encoded() {
        return Err(filesystem_catalog_artifact::invalid_data(
            FilesystemCatalogPublicationError::ByteConflict {
                artifact: CatalogRestartArtifact::Head,
            },
        ));
    }
    if observed.generation() == expected.generation()
        && observed.catalog_digest() == expected.catalog_digest()
    {
        Ok(())
    } else {
        Err(filesystem_catalog_artifact::invalid_data(
            FilesystemCatalogPublicationError::CurrentState {
                expected_generation: Some(expected.generation()),
                expected_digest: Some(expected.catalog_digest()),
                observed_generation: Some(observed.generation()),
                observed_digest: Some(observed.catalog_digest()),
            },
        ))
    }
}

pub(super) fn replace(stages: &FilesystemCatalogStages) -> io::Result<()> {
    stages.root.rename(
        filesystem_catalog_publisher::NEXT_HEAD,
        &stages.root,
        filesystem_catalog_publisher::HEAD,
    )
}

pub(super) fn synchronize_root(stages: &FilesystemCatalogStages) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&stages.root)
}

fn stage_mut(stages: &mut FilesystemCatalogStages) -> io::Result<&mut cap_std::fs::File> {
    stages.head_stage.as_mut().ok_or_else(stage_state)
}

fn stage_state() -> io::Error {
    filesystem_catalog_artifact::invalid_data(FilesystemCatalogPublicationError::StageState {
        artifact: CatalogRestartArtifact::Head,
    })
}
