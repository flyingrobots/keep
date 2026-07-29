//! This module owns writer-locked current-head verification.

use std::io;

use super::{
    CatalogPublicationExpectation, CatalogPublicationReadiness, CatalogRestartError,
    CatalogRestartPhase, CatalogSnapshot, FilesystemCatalogPublicationError,
    FilesystemCatalogPublisher, SegmentPublication, catalog_restart_loader,
    filesystem_catalog_artifact, filesystem_catalog_publisher,
};

pub(super) fn verify(
    publisher: &FilesystemCatalogPublisher,
    expected: CatalogPublicationExpectation,
    candidate: &CatalogSnapshot<'_, '_, '_>,
    segment: &SegmentPublication<'_, '_>,
) -> io::Result<CatalogPublicationReadiness> {
    require_no_next_head(publisher)?;
    require_no_catalog_stage(publisher)?;
    if segment.admitted().is_none() {
        require_no_segment_stage(publisher)?;
    }
    let readiness = match catalog_restart_loader::load_from_directory(
        &publisher.root,
        filesystem_catalog_publisher::HEAD,
        publisher.policy,
    ) {
        Ok(observed) => {
            let observed_generation = Some(observed.generation());
            let observed_digest = Some(observed.catalog_digest());
            if expected.current_generation() == observed_generation
                && expected.current_catalog_digest() == observed_digest
            {
                Ok(CatalogPublicationReadiness::Ready)
            } else if observed.generation() == candidate.generation()
                && observed.catalog_digest() == candidate.catalog_digest()
            {
                Ok(CatalogPublicationReadiness::AlreadyPublished)
            } else {
                Err(filesystem_catalog_artifact::invalid_data(
                    FilesystemCatalogPublicationError::CurrentState {
                        expected_generation: expected.current_generation(),
                        expected_digest: expected.current_catalog_digest(),
                        observed_generation,
                        observed_digest,
                    },
                ))
            }
        }
        Err(source) if head_is_absent(&source) && expected.current_generation().is_none() => {
            Ok(CatalogPublicationReadiness::Ready)
        }
        Err(source) => Err(filesystem_catalog_artifact::invalid_data(source)),
    }?;
    if readiness == CatalogPublicationReadiness::AlreadyPublished {
        require_no_segment_stage(publisher)?;
    }
    Ok(readiness)
}

fn require_no_next_head(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    match publisher
        .root
        .symlink_metadata(filesystem_catalog_publisher::NEXT_HEAD)
    {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(source),
        Ok(_metadata) => Err(filesystem_catalog_artifact::invalid_data(
            FilesystemCatalogPublicationError::HeadRecoveryRequired,
        )),
    }
}

fn require_no_catalog_stage(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    require_absent(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_CATALOG,
        FilesystemCatalogPublicationError::CatalogRecoveryRequired,
    )
}

fn require_no_segment_stage(publisher: &FilesystemCatalogPublisher) -> io::Result<()> {
    require_absent(
        &publisher.staging,
        filesystem_catalog_publisher::CURRENT_SEGMENT,
        FilesystemCatalogPublicationError::SegmentRecoveryRequired,
    )
}

fn require_absent(
    directory: &cap_std::fs::Dir,
    name: &str,
    error: FilesystemCatalogPublicationError,
) -> io::Result<()> {
    match directory.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(source),
        Ok(_metadata) => Err(filesystem_catalog_artifact::invalid_data(error)),
    }
}

fn head_is_absent(error: &CatalogRestartError) -> bool {
    matches!(
        error,
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenHead,
            source,
        } if source.kind() == io::ErrorKind::NotFound
    )
}
