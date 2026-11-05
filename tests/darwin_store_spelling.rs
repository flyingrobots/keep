//! A respelled protocol name never admits a complete store on either APFS variant.
//!
//! Size: medium. Oracle: a case-only rename leaves every byte intact. Namespace
//! admission refuses the foreign entry from its directory listing, independently
//! of the per-lookup spelling check, which `darwin_filesystem` tests pin directly.
//! Delete when public laws over a shared platform corpus subsume it.

#![cfg(all(feature = "repository-tasks", target_os = "macos"))]

#[allow(
    dead_code,
    reason = "these laws use only the initialized-store fixture"
)]
#[path = "reader_fence_process/fixture.rs"]
pub mod fixture;
#[allow(
    dead_code,
    reason = "this uses only the fixture's subset of the builder"
)]
#[path = "support/populated_store.rs"]
pub mod populated_store;
#[allow(dead_code, reason = "these laws use Drop cleanup")]
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;

use std::{error::Error, fs, path::Path};

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
    FilesystemStoreAdmission, ReaderAttemptLimit, SegmentReadPolicy, StoreExpectation,
    StoreInitializationError, StoreInitializationRefusal,
};

fn load(root: &Path) -> Result<FilesystemRetentionSnapshot, Box<dyn Error>> {
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    Ok(FilesystemRetentionSnapshot::load(
        root,
        policy,
        ReaderAttemptLimit::DEFAULT,
        keep::StoreExpectation::NoExpectation,
    )?)
}

fn assert_respelling_refuses(law: &str, stored: &str, foreign: &str) -> Result<(), Box<dyn Error>> {
    let (store, writer) = fixture::initialized(law)?;
    drop(writer);
    drop(load(store.path())?);
    fs::rename(store.path().join(stored), store.path().join(foreign))?;
    assert!(
        load(store.path()).is_err(),
        "{foreign} answered for {stored}"
    );
    assert!(
        FilesystemStoreAdmission::reopen(store.path(), StoreExpectation::NoExpectation).is_err(),
        "{foreign} gave the writer authority for {stored}"
    );
    fs::rename(store.path().join(foreign), store.path().join(stored))?;
    drop(load(store.path())?);
    Ok(())
}

#[test]
fn a_respelled_head_refuses_until_its_exact_name_returns() -> Result<(), Box<dyn Error>> {
    assert_respelling_refuses("darwin-respelled-head", "HEAD", "head")
}

#[test]
fn a_respelled_format_marker_refuses_until_its_exact_name_returns() -> Result<(), Box<dyn Error>> {
    assert_respelling_refuses("darwin-respelled-format", "FORMAT", "format")
}

#[test]
fn a_respelled_segment_pool_refuses_until_its_exact_name_returns() -> Result<(), Box<dyn Error>> {
    assert_respelling_refuses("darwin-respelled-pool", "segments", "Segments")
}

#[test]
fn initialization_refuses_a_foreign_spelling_and_leaves_it_in_place() -> Result<(), Box<dyn Error>>
{
    let sandbox = sandbox::TestDirectory::create("darwin-initialization-alias")?;
    fs::write(sandbox.path().join("Format"), b"foreign")?;
    let refused = FilesystemStoreAdmission::initialize(sandbox.path());
    assert!(matches!(
        refused,
        Err(StoreInitializationError::Refused {
            refusal: StoreInitializationRefusal::NotAPrefix { .. }
        })
    ));
    assert_eq!(fs::read(sandbox.path().join("Format"))?, b"foreign");
    assert_eq!(fs::read_dir(sandbox.path())?.count(), 1);
    Ok(())
}

// Finder can write this into any folder it displays. Refusal is deliberate until
// an owner decides otherwise: the namespace admits no entry it cannot account for.
#[test]
fn a_finder_metadata_file_in_the_root_refuses_until_it_is_removed() -> Result<(), Box<dyn Error>> {
    let (store, writer) = fixture::initialized("darwin-finder-metadata")?;
    drop(writer);
    let metadata = store.path().join(".DS_Store");
    fs::write(&metadata, [])?;
    assert!(
        load(store.path()).is_err(),
        "a foreign root entry was admitted"
    );
    fs::remove_file(&metadata)?;
    drop(load(store.path())?);
    Ok(())
}
