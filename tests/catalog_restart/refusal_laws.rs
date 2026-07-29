//! Published restart corruption, dangling-state, and conflict laws.

use std::error::Error;
use std::fs;
use std::io::ErrorKind;

use keep::{
    CatalogDecodeError, CatalogRestartError, CatalogRestartPhase, FilesystemCatalogSnapshot,
    PublicationHeadDecodeError,
};

use super::{StoreFixture, fixture, restart_policy};
use crate::support::require_error;

const BUNDLE_CATALOG_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const EMPTY_SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/empty-segment.hex");
const HEAD_VERSION_OFFSET: usize = 17;
const CATALOG_FLAGS_OFFSET: usize = 19;

#[test]
fn corrupt_and_unsupported_heads_refuse_at_the_head_boundary() -> Result<(), Box<dyn Error>> {
    let corrupt = StoreFixture::create("restart-corrupt-head")?;
    let mut bytes = fs::read(corrupt.path().join("HEAD"))?;
    *bytes.last_mut().ok_or("head fixture is empty")? ^= 1;
    fs::write(corrupt.path().join("HEAD"), bytes)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(corrupt.path(), restart_policy()?),
        "corrupt head was loaded",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::Head {
            source: PublicationHeadDecodeError::ChecksumMismatch { .. }
        }
    ));
    corrupt.remove()?;

    let unsupported = StoreFixture::create("restart-unsupported-head")?;
    let mut bytes = fs::read(unsupported.path().join("HEAD"))?;
    *bytes
        .get_mut(HEAD_VERSION_OFFSET)
        .ok_or("head lacks version field")? = 2;
    fs::write(unsupported.path().join("HEAD"), bytes)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(unsupported.path(), restart_policy()?),
        "unsupported head was loaded",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::Head {
            source: PublicationHeadDecodeError::UnsupportedVersion { observed: 2, .. }
        }
    ));
    unsupported.remove()
}

#[test]
fn noncanonical_catalog_refuses_before_segment_loading() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("restart-noncanonical-catalog")?;
    let mut bytes = fs::read(&store.catalog_path)?;
    *bytes
        .get_mut(CATALOG_FLAGS_OFFSET)
        .ok_or("catalog lacks flags field")? = 1;
    fs::write(&store.catalog_path, bytes)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(store.path(), restart_policy()?),
        "noncanonical catalog was loaded",
    )?;

    assert!(matches!(
        error,
        CatalogRestartError::Catalog {
            source: CatalogDecodeError::Flags { observed: 1, .. }
        }
    ));
    store.remove()
}

#[test]
fn dangling_catalog_and_segment_paths_refuse_exactly() -> Result<(), Box<dyn Error>> {
    let missing_catalog = StoreFixture::create("restart-missing-catalog")?;
    fs::remove_file(&missing_catalog.catalog_path)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(missing_catalog.path(), restart_policy()?),
        "missing catalog was loaded",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenCatalog,
            ref source,
        } if source.kind() == ErrorKind::NotFound
    ));
    missing_catalog.remove()?;

    let missing_segment = StoreFixture::create("restart-missing-segment")?;
    fs::remove_file(&missing_segment.segment_path)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(missing_segment.path(), restart_policy()?),
        "missing segment was loaded",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenSegment,
            ref source,
        } if source.kind() == ErrorKind::NotFound
    ));
    missing_segment.remove()
}

#[test]
fn physical_name_content_conflicts_are_never_substituted() -> Result<(), Box<dyn Error>> {
    let catalog = StoreFixture::create("restart-conflicting-catalog")?;
    fs::write(&catalog.catalog_path, fixture(BUNDLE_CATALOG_HEX)?)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(catalog.path(), restart_policy()?),
        "wrong catalog bytes were substituted under the selected name",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::CatalogCoordinate { .. }
    ));
    catalog.remove()?;

    let segment = StoreFixture::create("restart-conflicting-segment")?;
    fs::write(&segment.segment_path, fixture(EMPTY_SEGMENT_HEX)?)?;
    let error = require_error(
        FilesystemCatalogSnapshot::load(segment.path(), restart_policy()?),
        "wrong segment bytes were substituted under the selected name",
    )?;
    assert!(matches!(
        error,
        CatalogRestartError::SegmentCoordinate { .. }
    ));
    segment.remove()
}
