//! This module owns the crash law for a catalog a caller recorded before publication.
//!
//! A caller that names generation one in its `StoreExpectation` and dies during
//! a publication cannot know whether `HEAD` moved. The store must admit that
//! caller while generation one is published and refuse it, reporting what it
//! holds, once generation two is. Keep cannot tell the caller's own successor
//! from another store; accepting the observed catalog is the caller's decision.

use std::io;
use std::path::Path;

use keep::{
    CatalogRestartError, ChecksummedPublicationHead, ExpectedCatalog, FilesystemCatalogSnapshot,
    StoreExpectation,
};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::fixture::GoldenFixture;
use super::super::production_protocol::retained_catalog::policy;
use super::super::production_protocol::verification;

/// The generation-one catalog of the bundle store, as a caller would have recorded it.
pub(super) fn generation_one() -> Result<ExpectedCatalog, DurabilityCrashMatrixError> {
    let fixture = GoldenFixture::bundle_head()?;
    let head = ChecksummedPublicationHead::decode(fixture.bytes())
        .map_err(|source| verification("decode recorded generation-one head", source))?;
    Ok(ExpectedCatalog::new(
        head.generation(),
        head.catalog_digest(),
    ))
}

/// Requires the recorded generation-one catalog to refuse against generation two.
pub(super) fn require_refused_at_generation_two(
    store_root: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
    let recorded = generation_one()?;
    let loaded =
        FilesystemCatalogSnapshot::load(store_root, policy()?, StoreExpectation::Catalog(recorded));
    match loaded {
        Err(CatalogRestartError::Expectation { source })
            if source.expected() == recorded
                && source.observed().map(|found| found.generation().get()) == Some(2) =>
        {
            Ok(())
        }
        Err(source) => Err(verification(
            "refuse the recorded generation-one catalog",
            source,
        )),
        Ok(_admitted) => Err(verification(
            "refuse the recorded generation-one catalog",
            io::Error::other("generation two admitted a generation-one expectation"),
        )),
    }
}
