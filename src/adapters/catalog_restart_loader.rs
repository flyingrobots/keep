//! This module owns capability-relative published catalog restart loading.

use std::path::Path;

use crate::CatalogLength;

use super::{
    CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy,
    ChecksummedCatalog, ChecksummedPublicationHead, FilesystemCatalogSnapshot, ObservedCatalog,
    StoreExpectation, catalog_restart_io, catalog_restart_segments, filesystem_store_admission,
    filesystem_store_namespace, physical_pool_name,
};

const HEAD_NAME: &str = "HEAD";
const CATALOGS_NAME: &str = "catalogs";
const SEGMENTS_NAME: &str = "segments";
const HEAD_LENGTH: u64 = 128;

/// Opens `root`, admits it as a committed store, and loads the `HEAD` snapshot.
///
/// Admission is the reader law: the exact namespace, which may hold an
/// in-flight `head.next`, and the canonical `FORMAT` marker. A root without the
/// marker refuses before any catalog artifact is opened. `expectation` is
/// compared with what the checksummed `HEAD` names, or with its absence.
pub(super) fn load(
    root: &Path,
    policy: CatalogRestartPolicy,
    expectation: StoreExpectation,
) -> Result<FilesystemCatalogSnapshot, CatalogRestartError> {
    let directory = catalog_restart_io::open_root(root)?;
    filesystem_store_admission::admit_root(
        &directory,
        filesystem_store_namespace::admit_store_with_catalog_stage,
    )
    .map_err(|source| CatalogRestartError::Admission {
        source: Box::new(source),
    })?;
    match load_expected(&directory, HEAD_NAME, policy, expectation) {
        Err(source) if missing_head(&source) => {
            require(expectation, None)?;
            Err(source)
        }
        loaded => loaded,
    }
}

fn require(
    expectation: StoreExpectation,
    observed: Option<ObservedCatalog>,
) -> Result<(), CatalogRestartError> {
    expectation
        .require(observed)
        .map_err(|source| CatalogRestartError::Expectation { source })
}

/// Whether `error` is exactly the absence of the published head.
pub(super) fn missing_head(error: &CatalogRestartError) -> bool {
    matches!(
        error,
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenHead,
            source,
        } if source.kind() == std::io::ErrorKind::NotFound
    )
}

/// Reads the published catalog's coordinates from `HEAD`, or `None` when none is published.
///
/// The checksummed head names the catalog; this neither opens nor verifies it.
pub(super) fn observe_published_catalog(
    directory: &crate::adapters::filesystem_directory::FilesystemDirectory,
) -> Result<Option<ObservedCatalog>, CatalogRestartError> {
    match directory.symlink_metadata(HEAD_NAME) {
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CatalogRestartError::io(
                CatalogRestartPhase::OpenHead,
                source,
            ));
        }
        Ok(_) => {}
    }
    let head_bytes = read_head_bytes(directory, HEAD_NAME)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)
        .map_err(|source| CatalogRestartError::Head { source })?;
    Ok(Some(ObservedCatalog::new(
        head.generation(),
        head.catalog_digest(),
    )))
}

fn read_head_bytes(
    directory: &crate::adapters::filesystem_directory::FilesystemDirectory,
    head_name: &str,
) -> Result<Vec<u8>, CatalogRestartError> {
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
    catalog_restart_io::read_exact(
        head_file,
        CatalogRestartArtifact::Head,
        CatalogRestartPhase::ReadHead,
        HEAD_LENGTH,
    )
}

pub(super) fn load_from_directory(
    directory: &crate::adapters::filesystem_directory::FilesystemDirectory,
    head_name: &str,
    policy: CatalogRestartPolicy,
) -> Result<FilesystemCatalogSnapshot, CatalogRestartError> {
    load_expected(
        directory,
        head_name,
        policy,
        StoreExpectation::NoExpectation,
    )
}

/// Loads the snapshot `head_name` selects, comparing `expectation` with the head first.
///
/// The comparison runs on the checksummed head, before the catalog or any
/// segment is read. A store that publishes another catalog therefore refuses as
/// a mismatch whatever its catalog and segments hold, and a damaged head
/// refuses at its own boundary before the comparison.
fn load_expected(
    directory: &crate::adapters::filesystem_directory::FilesystemDirectory,
    head_name: &str,
    policy: CatalogRestartPolicy,
    expectation: StoreExpectation,
) -> Result<FilesystemCatalogSnapshot, CatalogRestartError> {
    let head_bytes = read_head_bytes(directory, head_name)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)
        .map_err(|source| CatalogRestartError::Head { source })?;
    let observed = ObservedCatalog::new(head.generation(), head.catalog_digest());
    require(expectation, Some(observed))?;
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
