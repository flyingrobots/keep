//! This module owns capability-relative published catalog restart loading.

use std::path::Path;

use cap_fs_ext::DirExt;

use crate::CatalogLength;

use super::{
    CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy,
    ChecksummedCatalog, ChecksummedPublicationHead, FilesystemCatalogSnapshot, catalog_restart_io,
    catalog_restart_segments, physical_pool_name,
};

const HEAD_NAME: &str = "HEAD";
const CATALOGS_NAME: &str = "catalogs";
const SEGMENTS_NAME: &str = "segments";
const HEAD_LENGTH: u64 = 128;

pub(super) fn load(
    root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<FilesystemCatalogSnapshot, CatalogRestartError> {
    let directory = catalog_restart_io::open_root(root)?;
    load_from_directory(&directory, HEAD_NAME, policy)
}

pub(super) fn load_from_directory(
    directory: &cap_std::fs::Dir,
    head_name: &str,
    policy: CatalogRestartPolicy,
) -> Result<FilesystemCatalogSnapshot, CatalogRestartError> {
    let (head_file, observed_head_length) = catalog_restart_io::open_regular(
        directory,
        head_name,
        CatalogRestartArtifact::Head,
        CatalogRestartPhase::OpenHead,
    )?;
    if observed_head_length != HEAD_LENGTH {
        return Err(CatalogRestartError::Length {
            artifact: CatalogRestartArtifact::Head,
            minimum: HEAD_LENGTH,
            maximum: HEAD_LENGTH,
            observed: observed_head_length,
        });
    }
    let head_bytes = catalog_restart_io::read_exact(
        head_file,
        CatalogRestartArtifact::Head,
        CatalogRestartPhase::ReadHead,
        HEAD_LENGTH,
    )?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)
        .map_err(|source| CatalogRestartError::Head { source })?;
    let catalogs = directory
        .open_dir_nofollow(CATALOGS_NAME)
        .map_err(|source| {
            CatalogRestartError::io(CatalogRestartPhase::OpenCatalogDirectory, source)
        })?;
    let catalog_name = physical_pool_name::catalog(head.generation(), head.catalog_digest());
    let (catalog_file, observed_catalog_length) = catalog_restart_io::open_regular(
        &catalogs,
        &catalog_name,
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::OpenCatalog,
    )?;
    if CatalogLength::new(observed_catalog_length).is_err() {
        return Err(CatalogRestartError::Length {
            artifact: CatalogRestartArtifact::Catalog,
            minimum: CatalogLength::MINIMUM.get(),
            maximum: CatalogLength::MAXIMUM.get(),
            observed: observed_catalog_length,
        });
    }
    let catalog_bytes = catalog_restart_io::read_exact(
        catalog_file,
        CatalogRestartArtifact::Catalog,
        CatalogRestartPhase::ReadCatalog,
        observed_catalog_length,
    )?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)
        .map_err(|source| CatalogRestartError::Catalog { source })?;
    validate_catalog_coordinate(head, catalog)?;
    let segment_digests = catalog_restart_segments::collect(catalog)?;
    let segments_directory = directory
        .open_dir_nofollow(SEGMENTS_NAME)
        .map_err(|source| {
            CatalogRestartError::io(CatalogRestartPhase::OpenSegmentDirectory, source)
        })?;
    let segments = catalog_restart_segments::load(&segments_directory, &segment_digests, policy)?;
    FilesystemCatalogSnapshot::admit(head_bytes, catalog_bytes, segments, policy)
}

fn validate_catalog_coordinate(
    head: ChecksummedPublicationHead<'_>,
    catalog: ChecksummedCatalog<'_>,
) -> Result<(), CatalogRestartError> {
    let generation_matches = head.generation() == catalog.generation();
    let length_matches = head.catalog_length() == catalog.length();
    let digest_matches = head.catalog_digest() == catalog.digest();
    if generation_matches && length_matches && digest_matches {
        Ok(())
    } else {
        Err(CatalogRestartError::CatalogCoordinate {
            expected_generation: head.generation(),
            observed_generation: catalog.generation(),
            expected_length: head.catalog_length(),
            observed_length: catalog.length(),
            expected_digest: head.catalog_digest(),
            observed_digest: catalog.digest(),
        })
    }
}
