//! This module owns exact staged and pooled publication artifact operations.

use std::error::Error;
use std::io;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, OpenOptions};

use super::segment_header::MAXIMUM_SEGMENT_LENGTH;
use super::{
    AdmittedSegment, CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase,
    ChecksummedCatalog, FilesystemCatalogPublicationError, SegmentReadPolicy, catalog_restart_io,
    sync_capable_directory,
};
use crate::CatalogLength;

pub(super) fn create_exclusive(directory: &Dir, name: &str) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    directory.open_with(name, &options)
}

pub(super) fn synchronize_directory(directory: &Dir) -> io::Result<()> {
    sync_capable_directory::open(directory, ".")?
        .into_std_file()
        .sync_all()
}

pub(super) fn link_without_replacement(
    source_directory: &Dir,
    source_name: &str,
    destination_directory: &Dir,
    destination_name: &str,
) -> io::Result<()> {
    match source_directory.hard_link(source_name, destination_directory, destination_name) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(source) => Err(source),
    }
}

pub(super) fn verify_segment(
    directory: &Dir,
    name: &str,
    expected: &AdmittedSegment<'_>,
    policy: SegmentReadPolicy,
) -> io::Result<()> {
    let artifact = CatalogRestartArtifact::Segment {
        digest: expected.digest(),
    };
    let (file, observed) = catalog_restart_io::open_regular(
        directory,
        name,
        artifact,
        CatalogRestartPhase::OpenSegment,
    )
    .map_err(invalid_data)?;
    if observed > MAXIMUM_SEGMENT_LENGTH {
        return Err(invalid_data(CatalogRestartError::Length {
            artifact,
            minimum: 0,
            maximum: MAXIMUM_SEGMENT_LENGTH,
            observed,
        }));
    }
    let encoded =
        catalog_restart_io::read_exact(file, artifact, CatalogRestartPhase::ReadSegment, observed)
            .map_err(invalid_data)?;
    let decoded = AdmittedSegment::decode(&encoded, policy).map_err(|source| {
        invalid_data(CatalogRestartError::Segment {
            expected: expected.digest(),
            source: Box::new(source),
        })
    })?;
    if decoded.digest() != expected.digest() {
        return Err(invalid_data(CatalogRestartError::SegmentCoordinate {
            expected: expected.digest(),
            observed: decoded.digest(),
        }));
    }
    require_exact_bytes(artifact, &encoded, expected.encoded())
}

pub(super) fn verify_catalog(
    directory: &Dir,
    name: &str,
    expected: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    let artifact = CatalogRestartArtifact::Catalog;
    let (file, observed) = catalog_restart_io::open_regular(
        directory,
        name,
        artifact,
        CatalogRestartPhase::OpenCatalog,
    )
    .map_err(invalid_data)?;
    if CatalogLength::new(observed).is_err() {
        return Err(invalid_data(CatalogRestartError::Length {
            artifact,
            minimum: CatalogLength::MINIMUM.get(),
            maximum: CatalogLength::MAXIMUM.get(),
            observed,
        }));
    }
    let encoded =
        catalog_restart_io::read_exact(file, artifact, CatalogRestartPhase::ReadCatalog, observed)
            .map_err(invalid_data)?;
    let decoded = ChecksummedCatalog::decode(&encoded)
        .map_err(|source| invalid_data(CatalogRestartError::Catalog { source }))?;
    require_catalog_coordinate(expected, decoded)?;
    require_exact_bytes(artifact, &encoded, expected.encoded())
}

pub(super) fn invalid_data(source: impl Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, source)
}

fn require_catalog_coordinate(
    expected: ChecksummedCatalog<'_>,
    observed: ChecksummedCatalog<'_>,
) -> io::Result<()> {
    let generation_matches = expected.generation() == observed.generation();
    let length_matches = expected.length() == observed.length();
    let digest_matches = expected.digest() == observed.digest();
    if generation_matches && length_matches && digest_matches {
        Ok(())
    } else {
        Err(invalid_data(CatalogRestartError::CatalogCoordinate {
            expected_generation: expected.generation(),
            observed_generation: observed.generation(),
            expected_length: expected.length(),
            observed_length: observed.length(),
            expected_digest: expected.digest(),
            observed_digest: observed.digest(),
        }))
    }
}

fn require_exact_bytes(
    artifact: CatalogRestartArtifact,
    observed: &[u8],
    expected: &[u8],
) -> io::Result<()> {
    if observed == expected {
        Ok(())
    } else {
        Err(invalid_data(
            FilesystemCatalogPublicationError::ByteConflict { artifact },
        ))
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::error::Error;

    use cap_std::{ambient_authority, fs::Dir};

    use super::super::filesystem_test_sandbox::TestDirectory;

    #[test]
    fn directory_synchronization_reopens_an_opath_capability() -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create("catalog-artifact-opath-sync")?;
        let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;

        super::synchronize_directory(&directory)?;

        drop(directory);
        sandbox.remove()?;
        Ok(())
    }
}
