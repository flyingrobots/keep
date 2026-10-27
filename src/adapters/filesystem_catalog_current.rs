//! This module owns writer-locked current-head verification.

use std::io;

use super::filesystem_catalog_stages::{self, FilesystemCatalogStages};
use super::{
    CatalogPublicationExpectation, CatalogPublicationReadiness, CatalogRestartError,
    CatalogRestartPhase, CatalogSnapshot, FilesystemCatalogPublicationError, SegmentPublication,
    catalog_restart_loader, filesystem_catalog_artifact,
};

pub(super) fn verify(
    stages: &FilesystemCatalogStages,
    expected: CatalogPublicationExpectation,
    candidate: &CatalogSnapshot<'_, '_, '_>,
    segment: &SegmentPublication<'_, '_>,
) -> io::Result<CatalogPublicationReadiness> {
    require_segment_authority(stages, segment)?;
    require_no_next_head(stages)?;
    require_no_catalog_stage(stages)?;
    if segment.admitted().is_none() {
        require_no_segment_stage(stages)?;
    }
    let readiness = match catalog_restart_loader::load_from_directory(
        &stages.root,
        filesystem_catalog_stages::HEAD,
        stages.policy,
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
            require_empty_durable_pools(stages)?;
            Ok(CatalogPublicationReadiness::Ready)
        }
        Err(source) => Err(filesystem_catalog_artifact::invalid_data(source)),
    }?;
    if readiness == CatalogPublicationReadiness::AlreadyPublished {
        require_no_segment_stage(stages)?;
    }
    Ok(readiness)
}

fn require_segment_authority(
    stages: &FilesystemCatalogStages,
    segment: &SegmentPublication<'_, '_>,
) -> io::Result<()> {
    if segment.is_bound_to(&stages.authority) {
        Ok(())
    } else {
        Err(filesystem_catalog_artifact::invalid_data(
            FilesystemCatalogPublicationError::SegmentAuthorityRequired,
        ))
    }
}

fn require_no_next_head(stages: &FilesystemCatalogStages) -> io::Result<()> {
    match stages
        .root
        .symlink_metadata(filesystem_catalog_stages::NEXT_HEAD)
    {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(source),
        Ok(_metadata) => Err(filesystem_catalog_artifact::invalid_data(
            FilesystemCatalogPublicationError::HeadRecoveryRequired,
        )),
    }
}

fn require_no_catalog_stage(stages: &FilesystemCatalogStages) -> io::Result<()> {
    require_absent(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_CATALOG,
        FilesystemCatalogPublicationError::CatalogRecoveryRequired,
    )
}

fn require_no_segment_stage(stages: &FilesystemCatalogStages) -> io::Result<()> {
    require_absent(
        &stages.staging,
        filesystem_catalog_stages::CURRENT_SEGMENT,
        FilesystemCatalogPublicationError::SegmentRecoveryRequired,
    )
}

fn require_empty_durable_pools(stages: &FilesystemCatalogStages) -> io::Result<()> {
    require_empty_pool(
        &stages.segments,
        FilesystemCatalogPublicationError::SegmentPoolRecoveryRequired,
    )?;
    require_empty_pool(
        &stages.catalogs,
        FilesystemCatalogPublicationError::CatalogPoolRecoveryRequired,
    )
}

fn require_empty_pool(
    directory: &cap_std::fs::Dir,
    error: FilesystemCatalogPublicationError,
) -> io::Result<()> {
    match directory.entries()?.next() {
        None => Ok(()),
        Some(Err(source)) => Err(source),
        Some(Ok(_entry)) => Err(filesystem_catalog_artifact::invalid_data(error)),
    }
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
