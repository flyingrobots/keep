//! This module owns the Golden File Worldline's sandboxed populated stores.
//!
//! Construction belongs to the shared `tests/support` builder; this module
//! owns only the sandbox and the worldline retention namespace.

use std::error::Error;

use super::durable_sandbox::TestDirectory;
use super::populated_store::{self, FirstCatalogRecords, StoreAdmission};
pub(super) use super::populated_store::{identify, policy};

type TestResult<T> = Result<T, Box<dyn Error>>;

/// A store holding `contents` with one "worldline" root that retains them all.
pub(super) fn build(name: &str, contents: &[&[u8]]) -> TestResult<TestDirectory> {
    let sandbox = created(name, contents, FirstCatalogRecords::Complete)?;
    let _receipt = populated_store::retain(
        sandbox.path(),
        StoreAdmission::PLATFORM,
        b"worldline",
        contents,
    )?;
    Ok(sandbox)
}

/// An unretained store whose catalog holds `content`'s layout but no chunk.
pub(super) fn build_missing_chunk(name: &str, content: &[u8]) -> TestResult<TestDirectory> {
    created(name, &[content], FirstCatalogRecords::LayoutsOnly)
}

/// An unretained store whose catalog holds `content`'s layout and only chunk `index`.
pub(super) fn build_selected_chunk(
    name: &str,
    content: &[u8],
    index: usize,
) -> TestResult<TestDirectory> {
    let selected = identify(content)?
        .spans
        .get(index)
        .ok_or("selected chunk absent")?
        .id();
    created(name, &[content], FirstCatalogRecords::OnlyChunk(selected))
}

fn created(
    name: &str,
    contents: &[&[u8]],
    selection: FirstCatalogRecords,
) -> TestResult<TestDirectory> {
    let sandbox = TestDirectory::create(name)?;
    let _created = populated_store::create(
        sandbox.path(),
        StoreAdmission::PLATFORM,
        contents,
        selection,
    )?;
    Ok(sandbox)
}
