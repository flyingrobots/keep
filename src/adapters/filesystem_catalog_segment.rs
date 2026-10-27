//! This module owns filesystem segment-pool publication transitions.

use std::io;

use super::filesystem_catalog_stages::{self, FilesystemCatalogStages};
use super::{AdmittedSegment, filesystem_catalog_artifact, physical_pool_name};

pub(super) fn link(
    stages: &FilesystemCatalogStages,
    segment: &AdmittedSegment<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_segment(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_SEGMENT,
        segment,
        stages.policy.segment_read(),
    )?;
    super::filesystem_exact_record::link_without_replacement(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_SEGMENT,
        &stages.segments,
        &physical_pool_name::segment(segment.digest()),
    )
}

pub(super) fn verify_pool(
    stages: &FilesystemCatalogStages,
    segment: &AdmittedSegment<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_segment(
        &stages.segments,
        &physical_pool_name::segment(segment.digest()),
        segment,
        stages.policy.segment_read(),
    )
}

pub(super) fn synchronize_pool(stages: &FilesystemCatalogStages) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&stages.segments)
}

pub(super) fn remove_stage(stages: &FilesystemCatalogStages) -> io::Result<()> {
    stages
        .staging
        .remove_file(filesystem_catalog_stages::CURRENT_SEGMENT)
}

pub(super) fn synchronize_staging(stages: &FilesystemCatalogStages) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&stages.staging)
}
