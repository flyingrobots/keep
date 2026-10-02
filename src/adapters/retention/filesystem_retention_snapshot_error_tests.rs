//! This module owns exact public reader refusal boundaries.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use super::filesystem_retention_test_fixture::{CATALOG_NAME, SEGMENT_NAME, migrated_store};
use super::{
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError as SnapshotError,
    ReaderAttemptLimit, RetentionViewError,
};
use crate::adapters::{
    CatalogDecodeError, CatalogRestartByteLimit, CatalogRestartError, CatalogRestartPhase,
    CatalogRestartPolicy, PublicationHeadDecodeError, SegmentReadPolicy,
};

// Size: medium (owned filesystem). Oracle: selected-artifact restart failures
// have the documented catalog boundary and retain their exact typed source.
// Delete when the reader API is removed or stronger public laws subsume these cases.
#[test]
fn a_missing_selected_catalog_reports_its_exact_catalog_refusal() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-error-missing-catalog")?;
    fs::remove_file(sandbox.path().join("catalogs").join(CATALOG_NAME))?;

    let error = refusal(sandbox.path())?;

    assert!(
        matches!(&error, Some(SnapshotError::Catalog {
            source: CatalogRestartError::Io { phase: CatalogRestartPhase::OpenCatalog, source }
        }) if source.kind() == io::ErrorKind::NotFound),
        "missing selected catalog must preserve its catalog boundary, phase and I/O kind: {error:?}"
    );
    assert!(
        error
            .as_ref()
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<CatalogRestartError>())
            .is_some(),
        "catalog refusal must expose CatalogRestartError directly as its source"
    );
    Ok(())
}

// Size: medium. Oracle and deletion criterion: the public catalog-boundary law above.
#[test]
fn a_missing_selected_segment_reports_its_exact_catalog_refusal() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-error-missing-segment")?;
    fs::remove_file(sandbox.path().join("segments").join(SEGMENT_NAME))?;

    let error = refusal(sandbox.path())?;

    assert!(
        matches!(&error, Some(SnapshotError::Catalog {
            source: CatalogRestartError::Io { phase: CatalogRestartPhase::OpenSegment, source }
        }) if source.kind() == io::ErrorKind::NotFound),
        "missing selected segment must preserve its catalog boundary, phase and I/O kind: {error:?}"
    );
    Ok(())
}

// Size: medium. Oracle and deletion criterion: the public catalog-boundary law above.
#[test]
fn a_corrupt_selected_catalog_reports_its_decoder_refusal() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-error-corrupt-catalog")?;
    let path = sandbox.path().join("catalogs").join(CATALOG_NAME);
    let mut bytes = fs::read(&path)?;
    let checksum = bytes
        .len()
        .checked_sub(64)
        .ok_or("catalog lacks checksum")?;
    *bytes.get_mut(checksum).ok_or("catalog checksum absent")? ^= 1;
    fs::write(path, bytes)?;

    let error = refusal(sandbox.path())?;

    assert!(
        matches!(
            error,
            Some(SnapshotError::Catalog {
                source: CatalogRestartError::Catalog {
                    source: CatalogDecodeError::ChecksumMismatch { .. }
                }
            })
        ),
        "corrupt selected catalog must preserve its exact decoder refusal: {error:?}"
    );
    Ok(())
}

// Size: medium. Oracle: coordinate decoding remains a view failure, not a catalog load.
// Delete when the coordinate-read contract is removed or a stronger law subsumes this case.
#[test]
fn a_corrupt_catalog_head_reports_its_exact_view_refusal() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-error-corrupt-head")?;
    let path = sandbox.path().join("HEAD");
    let mut bytes = fs::read(&path)?;
    *bytes.last_mut().ok_or("HEAD absent")? ^= 1;
    fs::write(path, bytes)?;

    let error = refusal(sandbox.path())?;

    assert!(
        matches!(&error, Some(SnapshotError::View {
            source: RetentionViewError::Io { source }
        }) if matches!(source.get_ref().and_then(|source| source.downcast_ref::<PublicationHeadDecodeError>()),
            Some(PublicationHeadDecodeError::ChecksumMismatch { .. }))),
        "corrupt coordinate HEAD must preserve its view boundary and decoder source: {error:?}"
    );
    Ok(())
}

fn refusal(path: &Path) -> Result<Option<SnapshotError>, Box<dyn Error>> {
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    Ok(FilesystemRetentionSnapshot::load(path, policy, ReaderAttemptLimit::DEFAULT).err())
}
