//! This module owns filesystem catalog-stage and pool transitions.

use std::io::{self, Write};

use super::filesystem_catalog_stages::{self, FilesystemCatalogStages};
use super::{
    CanonicalCatalog, CatalogRestartArtifact, ChecksummedCatalog,
    FilesystemCatalogPublicationError, filesystem_catalog_artifact, physical_pool_name,
};

pub(super) fn create_stage(stages: &mut FilesystemCatalogStages) -> io::Result<()> {
    if stages.catalog_stage.is_some() {
        return Err(stage_state());
    }
    stages.catalog_stage = Some(filesystem_catalog_artifact::create_exclusive(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_CATALOG,
    )?);
    Ok(())
}

pub(super) fn write(
    stages: &mut FilesystemCatalogStages,
    catalog: &CanonicalCatalog,
) -> io::Result<()> {
    write_bytes(stages, catalog.encoded())
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

pub(super) fn link(
    stages: &mut FilesystemCatalogStages,
    catalog: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    let stage = stages.catalog_stage.take().ok_or_else(stage_state)?;
    drop(stage);
    filesystem_catalog_artifact::verify_catalog(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_CATALOG,
        catalog,
    )?;
    super::filesystem_exact_record::link_without_replacement(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_CATALOG,
        &stages.catalogs,
        &physical_pool_name::catalog(catalog.generation(), catalog.digest()),
    )
}

pub(super) fn verify_pool(
    stages: &FilesystemCatalogStages,
    catalog: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_catalog(
        &stages.catalogs,
        &physical_pool_name::catalog(catalog.generation(), catalog.digest()),
        catalog,
    )
}

pub(super) fn synchronize_pool(stages: &FilesystemCatalogStages) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&stages.catalogs)
}

pub(super) fn remove_stage(stages: &FilesystemCatalogStages) -> io::Result<()> {
    stages
        .staging
        .remove_file(filesystem_catalog_stages::CURRENT_CATALOG)
}

pub(super) fn synchronize_staging(stages: &FilesystemCatalogStages) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&stages.staging)
}

fn stage_mut(stages: &mut FilesystemCatalogStages) -> io::Result<&mut cap_std::fs::File> {
    stages.catalog_stage.as_mut().ok_or_else(stage_state)
}

fn stage_state() -> io::Error {
    filesystem_catalog_artifact::invalid_data(FilesystemCatalogPublicationError::StageState {
        artifact: CatalogRestartArtifact::Catalog,
    })
}
