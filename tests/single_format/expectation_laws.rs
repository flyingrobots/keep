//! This module owns the ADR-0010 expected-state laws at every normal open.
//!
//! A restored store's records are all consistent, so only state the caller kept
//! outside the store can refuse it. Each law opens through the five production
//! entry points that take a `StoreExpectation` and asserts the exact expected
//! and observed catalogs. Recovery tools, verification loads, and initialization
//! take no expectation and are outside these laws.

use std::fs;
use std::path::Path;

use keep::{
    CatalogGeneration, CatalogRestartError, CatalogRestartPhase, DurableSnapshot, DurableStore,
    DurableStoreError, ExpectedCatalog, FilesystemCatalogSnapshot, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, FilesystemStoreAdmission, FilesystemStoreAdmissionError,
    ReaderAttemptLimit, StoreExpectation, StoreExpectationMismatch,
};

use super::durable_fixture;
use super::durable_sandbox::TestDirectory;
use super::populated_store;
use super::store_copy::copy_store;
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, publish, retain, second_blob,
};

/// The catalog a fresh view of `root` selects, as a caller would record it.
fn published(root: &Path) -> TestResult<ExpectedCatalog> {
    let snapshot = DurableSnapshot::open(
        root,
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
        StoreExpectation::NoExpectation,
    )?;
    let view = snapshot.view();
    Ok(ExpectedCatalog::new(
        view.catalog_generation(),
        view.catalog_digest(),
    ))
}

fn publish_second(root: &Path) -> TestResult<()> {
    let second = second_blob();
    let mut writer = open_publisher(root)?;
    let _receipt = publish(&mut writer, root, &second, Predecessor::Kept)??;
    drop(writer);
    retain(root, b"second", &second)
}

/// Requires every normal open of `root` to refuse with exactly this mismatch.
fn every_open_refuses(
    root: &Path,
    expected: ExpectedCatalog,
    observed: Option<ExpectedCatalog>,
) -> TestResult<()> {
    let expectation = StoreExpectation::Catalog(expected);
    let policy = populated_store::policy()?;
    let limit = ReaderAttemptLimit::DEFAULT;
    let reader = FilesystemRetentionSnapshot::load(root, policy, limit, expectation);
    let Err(FilesystemRetentionSnapshotError::Expectation { source }) = reader else {
        return Err("the reader did not refuse at the expectation boundary".into());
    };
    require_mismatch(source, expected, observed);
    let store = DurableStore::open(root, policy, limit, expectation)?;
    let Err(DurableStoreError::Snapshot(error)) = store.snapshot() else {
        return Err("the durable store did not refuse its snapshot".into());
    };
    let FilesystemRetentionSnapshotError::Expectation { source } = *error else {
        return Err("the durable store lost the expectation boundary".into());
    };
    require_mismatch(source, expected, observed);
    let writer = FilesystemStoreAdmission::reopen(root, expectation);
    let Err(FilesystemStoreAdmissionError::Expectation { source }) = writer else {
        return Err("the writer did not refuse at the expectation boundary".into());
    };
    require_mismatch(source, expected, observed);
    let catalog = FilesystemCatalogSnapshot::load(root, policy, expectation);
    let Err(CatalogRestartError::Expectation { source }) = catalog else {
        return Err("the catalog snapshot did not refuse at the expectation boundary".into());
    };
    require_mismatch(source, expected, observed);
    Ok(())
}

fn require_mismatch(
    mismatch: StoreExpectationMismatch,
    expected: ExpectedCatalog,
    observed: Option<ExpectedCatalog>,
) {
    assert_eq!(mismatch.expected(), expected);
    let found = mismatch
        .observed()
        .map(|found| ExpectedCatalog::new(found.generation(), found.digest()));
    assert_eq!(found, observed);
}

fn every_open_admits(root: &Path, expectation: StoreExpectation) -> TestResult<()> {
    let policy = populated_store::policy()?;
    let limit = ReaderAttemptLimit::DEFAULT;
    drop(FilesystemRetentionSnapshot::load(
        root,
        policy,
        limit,
        expectation,
    )?);
    drop(DurableStore::open(root, policy, limit, expectation)?.snapshot()?);
    drop(FilesystemStoreAdmission::reopen(root, expectation)?);
    drop(FilesystemCatalogSnapshot::load(root, policy, expectation)?);
    Ok(())
}

/// Replaces every entry under `root` with the entries of `snapshot`.
fn restore(snapshot: &Path, root: &Path) -> TestResult<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    copy_store(snapshot, root)
}

#[test]
fn an_older_snapshot_restored_in_place_refuses_newer_expected_state() -> TestResult<()> {
    let first = first_blob();
    let store = durable_fixture::build("expected-rollback", &[&first])?;
    let snapshot = TestDirectory::create("expected-rollback-snapshot")?;
    copy_store(store.path(), snapshot.path())?;
    let older = published(store.path())?;
    publish_second(store.path())?;
    let newer = published(store.path())?;
    assert_ne!(older, newer);
    every_open_admits(store.path(), StoreExpectation::Catalog(newer))?;
    every_open_refuses(store.path(), older, Some(newer))?;

    restore(snapshot.path(), store.path())?;

    every_open_refuses(store.path(), newer, Some(older))?;
    // The limit the typed choice exists to make visible: without an expectation,
    // nothing in a consistent restored store shows that newer history existed.
    every_open_admits(store.path(), StoreExpectation::NoExpectation)?;
    assert_reads(store.path(), &first)
}

#[test]
fn a_different_valid_store_refuses_the_expected_catalog() -> TestResult<()> {
    let intended = durable_fixture::build("expected-intended", &[&first_blob()])?;
    let other = durable_fixture::build("expected-other", &[&second_blob()])?;
    let wanted = published(intended.path())?;
    let found = published(other.path())?;
    assert_eq!(wanted.generation(), found.generation());

    every_open_refuses(other.path(), wanted, Some(found))
}

#[test]
fn a_store_with_no_catalog_refuses_an_expected_catalog() -> TestResult<()> {
    let published_store = durable_fixture::build("expected-published", &[&first_blob()])?;
    let wanted = published(published_store.path())?;
    let empty = TestDirectory::create("expected-unpublished")?;
    drop(FilesystemStoreAdmission::initialize(empty.path())?);

    every_open_refuses(empty.path(), wanted, None)?;
    assert!(matches!(
        FilesystemRetentionSnapshot::load(
            empty.path(),
            populated_store::policy()?,
            ReaderAttemptLimit::DEFAULT,
            StoreExpectation::NoExpectation,
        ),
        Err(FilesystemRetentionSnapshotError::Unpublished)
    ));
    // The catalog snapshot has nothing to load, and says so at the head boundary.
    assert!(matches!(
        FilesystemCatalogSnapshot::load(
            empty.path(),
            populated_store::policy()?,
            StoreExpectation::NoExpectation,
        ),
        Err(CatalogRestartError::Io { phase: CatalogRestartPhase::OpenHead, source })
            if source.kind() == std::io::ErrorKind::NotFound
    ));
    drop(FilesystemStoreAdmission::reopen(
        empty.path(),
        StoreExpectation::NoExpectation,
    )?);
    Ok(())
}

#[test]
fn a_damaged_head_refuses_the_writer_expectation_at_its_own_boundary() -> TestResult<()> {
    let store = durable_fixture::build("expected-damaged-head", &[&first_blob()])?;
    let wanted = published(store.path())?;
    let head = store.path().join("HEAD");
    let mut bytes = fs::read(&head)?;
    *bytes.first_mut().ok_or("empty HEAD")? ^= 1;
    fs::write(&head, &bytes)?;

    let refused = FilesystemStoreAdmission::reopen(store.path(), StoreExpectation::Catalog(wanted));

    assert!(matches!(
        refused,
        Err(FilesystemStoreAdmissionError::ExpectationHead { .. })
    ));
    Ok(())
}

/// Pins the documented limit: this expectation covers the catalog, so restoring
/// only older retention state under an unchanged catalog is not detected.
#[test]
fn a_catalog_expectation_does_not_detect_a_retention_only_rollback() -> TestResult<()> {
    let first = first_blob();
    let store = durable_fixture::build("expected-retention-only", &[&first])?;
    let snapshot = TestDirectory::create("expected-retention-only-snapshot")?;
    copy_store(store.path(), snapshot.path())?;
    let wanted = published(store.path())?;
    retain(store.path(), b"later", &first)?;
    assert_eq!(published(store.path())?, wanted);

    fs::remove_dir_all(store.path().join("retention"))?;
    fs::create_dir(store.path().join("retention"))?;
    copy_store(
        &snapshot.path().join("retention"),
        &store.path().join("retention"),
    )?;

    every_open_admits(store.path(), StoreExpectation::Catalog(wanted))
}

/// A later process holds only what the caller recorded: a `u64` and 32 bytes.
/// The expectation rebuilt from them admits the recorded catalog and refuses
/// its successor through every normal open.
#[test]
fn an_expectation_rebuilt_from_recorded_values_admits_and_refuses_as_the_original() -> TestResult<()>
{
    let store = durable_fixture::build("expected-recorded", &[&first_blob()])?;
    let original = published(store.path())?;
    let recorded: (u64, [u8; 32]) = (original.generation().get(), original.digest_bytes());

    let rebuilt = ExpectedCatalog::from_recorded(CatalogGeneration::new(recorded.0)?, recorded.1);

    assert_eq!(rebuilt, original);
    every_open_admits(store.path(), StoreExpectation::Catalog(rebuilt))?;
    publish_second(store.path())?;
    let successor = published(store.path())?;
    every_open_refuses(store.path(), rebuilt, Some(successor))?;
    // The right generation with one digest bit wrong must refuse too.
    let mut near = successor.digest_bytes();
    *near.first_mut().ok_or("empty digest")? ^= 1;
    let wrong = ExpectedCatalog::from_recorded(successor.generation(), near);
    every_open_refuses(store.path(), wrong, Some(successor))
}

/// The catalog snapshot compares `HEAD` before it opens what `HEAD` selects, so
/// a store that publishes another catalog refuses as a mismatch even when that
/// catalog is absent, exactly as the writer does.
#[test]
fn a_mismatch_refuses_the_catalog_snapshot_before_its_catalog_is_read() -> TestResult<()> {
    let intended = durable_fixture::build("expected-precedence-intended", &[&first_blob()])?;
    let other = durable_fixture::build("expected-precedence-other", &[&second_blob()])?;
    let wanted = published(intended.path())?;
    let found = published(other.path())?;
    for entry in fs::read_dir(other.path().join("catalogs"))? {
        fs::remove_file(entry?.path())?;
    }
    let policy = populated_store::policy()?;

    let mismatched =
        FilesystemCatalogSnapshot::load(other.path(), policy, StoreExpectation::Catalog(wanted));
    let unchecked =
        FilesystemCatalogSnapshot::load(other.path(), policy, StoreExpectation::NoExpectation);

    let Err(CatalogRestartError::Expectation { source }) = mismatched else {
        return Err("a mismatch was not reported before the absent catalog".into());
    };
    require_mismatch(source, wanted, Some(found));
    assert!(matches!(
        unchecked,
        Err(CatalogRestartError::Io { phase: CatalogRestartPhase::OpenCatalog, source })
            if source.kind() == std::io::ErrorKind::NotFound
    ));
    Ok(())
}
