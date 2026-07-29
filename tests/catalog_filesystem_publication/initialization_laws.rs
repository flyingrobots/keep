//! This module owns absent-head immutable-pool admission laws.

use std::error::Error;
use std::fs;
use std::io;

use keep::{
    CanonicalCatalog, CatalogGeneration, CatalogPublicationError, CatalogPublicationExpectation,
    CatalogPublicationPhase, FilesystemCatalogPublicationError, FilesystemCatalogPublisher,
    FilesystemWriterLock, SegmentPublication, publish_catalog_generation,
};

use super::{CATALOG_HEX, SEGMENT_HEX, StoreFixture, fixture, restart_policy};

#[test]
fn absent_head_refuses_a_retained_segment() -> Result<(), Box<dyn Error>> {
    require_empty_durable_pools(DurablePool::Segments)
}

#[test]
fn absent_head_refuses_a_retained_catalog() -> Result<(), Box<dyn Error>> {
    require_empty_durable_pools(DurablePool::Catalogs)
}

fn require_empty_durable_pools(pool: DurablePool) -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create(pool.fixture_name())?;
    let artifact = pool.write(&store)?;
    let segments = [];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;

    let Err(error) = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        SegmentPublication::none(),
        &catalog,
        &segments,
    ) else {
        return Err(format!("absent HEAD admitted retained {} bytes", pool.name()).into());
    };
    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("retained pool artifact reached the wrong refusal boundary".into());
    };

    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    let exact = source
        .get_ref()
        .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>());
    assert!(pool.matches(exact));
    drop(publisher);
    assert_eq!(fs::read(&artifact)?, pool.bytes()?);
    assert!(!store.path().join("HEAD").exists());
    assert!(!store.path().join("head.next").exists());
    assert!(!store.staging().join("current.cat").exists());
    assert!(!store.staging().join("current.seg").exists());
    store.remove()
}

#[derive(Clone, Copy)]
enum DurablePool {
    Segments,
    Catalogs,
}

impl DurablePool {
    const fn fixture_name(self) -> &'static str {
        match self {
            Self::Segments => "catalog-filesystem-orphan-segment",
            Self::Catalogs => "catalog-filesystem-orphan-catalog",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Segments => "segment-pool",
            Self::Catalogs => "catalog-pool",
        }
    }

    const fn matches(self, error: Option<&FilesystemCatalogPublicationError>) -> bool {
        matches!(
            (self, error),
            (
                Self::Segments,
                Some(FilesystemCatalogPublicationError::SegmentPoolRecoveryRequired)
            ) | (
                Self::Catalogs,
                Some(FilesystemCatalogPublicationError::CatalogPoolRecoveryRequired)
            )
        )
    }

    fn write(self, store: &StoreFixture) -> Result<std::path::PathBuf, Box<dyn Error>> {
        let path = match self {
            Self::Segments => store.segment_path(),
            Self::Catalogs => store.catalog_path(),
        };
        fs::write(path, self.bytes()?)?;
        Ok(path.to_path_buf())
    }

    fn bytes(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Self::Segments => fixture(SEGMENT_HEX),
            Self::Catalogs => fixture(CATALOG_HEX),
        }
    }
}
