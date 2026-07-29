//! This module owns filesystem segment-pool publication transitions.

use std::io;

use super::{
    AdmittedSegment, FilesystemCatalogPublisher, filesystem_catalog_artifact,
    filesystem_catalog_publisher, physical_pool_name,
};

pub(super) fn link(
    publisher: &FilesystemCatalogPublisher,
    segment: &AdmittedSegment<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_segment(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_SEGMENT,
        segment,
        publisher.policy.segment_read(),
    )?;
    filesystem_catalog_artifact::link_without_replacement(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_SEGMENT,
        &publisher.segments,
        &physical_pool_name::segment(segment.digest()),
    )
}

pub(super) fn verify_pool(
    publisher: &FilesystemCatalogPublisher,
    segment: &AdmittedSegment<'_>,
) -> io::Result<()> {
    filesystem_catalog_artifact::verify_segment(
        &publisher.segments,
        &physical_pool_name::segment(segment.digest()),
        segment,
        publisher.policy.segment_read(),
    )
}

pub(super) fn synchronize_pool(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&publisher.segments)
}

pub(super) fn remove_stage(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    publisher
        .staging
        .remove_file(filesystem_catalog_publisher::CURRENT_SEGMENT)
}

pub(super) fn synchronize_staging(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(&publisher.staging)
}
