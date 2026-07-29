//! This module owns filesystem publication-head transitions.

use std::io::{self, Write};

use super::{
    CanonicalPublicationHead, CatalogRestartArtifact, CatalogSnapshot,
    FilesystemCatalogPublicationError, FilesystemCatalogPublisher, catalog_restart_loader,
    filesystem_catalog_artifact, filesystem_catalog_publisher,
};

pub(super) fn create_stage(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    if publisher.head_stage.is_some() {
        return Err(stage_state());
    }
    publisher.head_stage = Some(filesystem_catalog_artifact::create_exclusive(
        &publisher.root,
        filesystem_catalog_publisher::NEXT_HEAD,
    )?);
    Ok(())
}

pub(super) fn write(
    publisher: &mut FilesystemCatalogPublisher,
    head: &CanonicalPublicationHead,
) -> io::Result<()> {
    stage_mut(publisher)?.write_all(head.encoded())
}

pub(super) fn flush(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    stage_mut(publisher)?.flush()
}

pub(super) fn synchronize(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    stage_mut(publisher)?.sync_all()
}

pub(super) fn verify_view(
    publisher: &mut FilesystemCatalogPublisher,
    head: &CanonicalPublicationHead,
    expected: &CatalogSnapshot<'_, '_, '_>,
) -> io::Result<()> {
    let stage = publisher.head_stage.take().ok_or_else(stage_state)?;
    drop(stage);
    let observed = catalog_restart_loader::load_from_directory(
        &publisher.root,
        filesystem_catalog_publisher::NEXT_HEAD,
        publisher.policy,
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

pub(super) fn replace(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    publisher.root.rename(
        filesystem_catalog_publisher::NEXT_HEAD,
        &publisher.root,
        filesystem_catalog_publisher::HEAD,
    )
}

pub(super) fn synchronize_root(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&publisher.root)
}

fn stage_mut(publisher: &mut FilesystemCatalogPublisher) -> io::Result<&mut cap_std::fs::File> {
    publisher.head_stage.as_mut().ok_or_else(stage_state)
}

fn stage_state() -> io::Error {
    filesystem_catalog_artifact::invalid_data(FilesystemCatalogPublicationError::StageState {
        artifact: CatalogRestartArtifact::Head,
    })
}
