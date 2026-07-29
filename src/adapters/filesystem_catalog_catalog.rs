//! This module owns filesystem catalog-stage and pool transitions.

use std::io::{self, Write};

use super::{
    CanonicalCatalog, CatalogRestartArtifact, ChecksummedCatalog,
    FilesystemCatalogPublicationError, FilesystemCatalogPublisher, filesystem_catalog_artifact,
    filesystem_catalog_publisher, physical_pool_name,
};

pub(super) fn create_stage(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    if publisher.catalog_stage.is_some() {
        return Err(stage_state());
    }
    publisher.catalog_stage = Some(filesystem_catalog_artifact::create_exclusive(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_CATALOG,
    )?);
    Ok(())
}

pub(super) fn write(
    publisher: &mut FilesystemCatalogPublisher,
    catalog: &CanonicalCatalog,
) -> io::Result<()> {
    stage_mut(publisher)?.write_all(catalog.encoded())
}

pub(super) fn flush(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    stage_mut(publisher)?.flush()
}

pub(super) fn synchronize(publisher: &mut FilesystemCatalogPublisher) -> io::Result<()> {
    stage_mut(publisher)?.sync_all()
}

pub(super) fn link(
    publisher: &mut FilesystemCatalogPublisher,
    catalog: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    let stage = publisher.catalog_stage.take().ok_or_else(stage_state)?;
    drop(stage);
    filesystem_catalog_artifact::verify_catalog(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_CATALOG,
        catalog,
    )?;
    filesystem_catalog_artifact::link_without_replacement(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_CATALOG,
        &publisher.catalogs,
        &physical_pool_name::catalog(catalog.generation(), catalog.digest()),
    )
}

pub(super) fn verify_pool(
    publisher: &FilesystemCatalogPublisher,
    catalog: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_catalog(
        &publisher.catalogs,
        &physical_pool_name::catalog(catalog.generation(), catalog.digest()),
        catalog,
    )
}

pub(super) fn synchronize_pool(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&publisher.catalogs)
}

pub(super) fn remove_stage(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    publisher
        .staging
        .remove_file(filesystem_catalog_publisher::CURRENT_CATALOG)
}

pub(super) fn synchronize_staging(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&publisher.staging)
}

fn stage_mut(publisher: &mut FilesystemCatalogPublisher) -> io::Result<&mut cap_std::fs::File> {
    publisher.catalog_stage.as_mut().ok_or_else(stage_state)
}

fn stage_state() -> io::Error {
    filesystem_catalog_artifact::invalid_data(FilesystemCatalogPublicationError::StageState {
        artifact: CatalogRestartArtifact::Catalog,
    })
}
