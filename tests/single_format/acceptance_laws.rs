//! This module owns the integrated acceptance sequence and the backup round trip.
//!
//! Both interrupt a real publication through the writer's repository-task
//! observer: `HEAD` replacement fails after `head.next` is synchronized,
//! which is exactly the residue process death there leaves. Recovery then
//! runs through the public, gated next-head finalizer.

use std::fs;
use std::io;
use std::path::Path;

use keep::{
    CatalogPublicationBoundary, CatalogPublicationPhase, DurableStore, FilesystemCatalogSnapshot,
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, FilesystemVersionTwoAdmission,
    FilesystemVersionTwoCatalogPublisher, ReaderAttemptLimit, RecoveryEntryRole,
    RecoveryNextHeadFinalizationOutcome, RetainedCatalogPublicationError,
    VersionTwoInitializationPrefix,
};

use super::durable_fixture;
use super::durable_sandbox::TestDirectory;
use super::populated_store;
use super::recovery_fixture::{finalize, stage_roles};
use super::store_copy::copy_store;
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, pool_segments, publish,
    retain, second_blob, witness,
};

/// Publishes `content` as the next generation and stops before `HEAD` replacement.
///
/// Returns the exact pooled segments the staged candidate references.
fn interrupted_publication(root: &Path, content: &[u8]) -> TestResult<Vec<Vec<u8>>> {
    let mut candidate = pool_segments(root)?;
    let mut publisher = open_publisher(root)?;
    publisher.observe_catalog_phases_for_repository_tasks(Box::new(|phase, boundary| {
        if phase == CatalogPublicationPhase::ReplaceHead
            && boundary == CatalogPublicationBoundary::Before
        {
            Err(io::Error::other("interrupted before HEAD replacement"))
        } else {
            Ok(())
        }
    }));
    let outcome = publish(&mut publisher, root, content, Predecessor::Kept)?;
    drop(publisher);
    if !matches!(
        outcome,
        Err(RetainedCatalogPublicationError::Catalog { .. })
    ) {
        return Err(format!("the interruption did not stop publication: {outcome:?}").into());
    }
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::NextHeadStage]);
    let mut pooled = pool_segments(root)?;
    pooled.retain(|segment| !candidate.contains(segment));
    let [staged] = <[Vec<u8>; 1]>::try_from(pooled)
        .map_err(|_| "the interrupted publication pooled other than one segment")?;
    candidate.push(staged);
    Ok(candidate)
}

/// Finalizes the staged head through the gate and requires the expected generation.
fn recover(root: &Path, candidate: &[Vec<u8>], generation: u64) -> TestResult<()> {
    let receipt = finalize(root, candidate)??;
    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(receipt.target().generation().get(), generation);
    assert!(stage_roles(root)?.is_empty());
    Ok(())
}

fn current_generation(root: &Path) -> TestResult<u64> {
    Ok(
        FilesystemCatalogSnapshot::load(root, populated_store::policy()?)?
            .generation()
            .get(),
    )
}

/// ADR-0010 step 4: initialize, ingest, reopen, reconstruct exact bytes,
/// retain, publish a second catalog, interrupt publication, recover, and read
/// both retained contents; a candidate that would lose retained content
/// refuses with its evidence intact.
#[test]
fn the_single_format_serves_every_operation_from_initialization_to_recovery() -> TestResult<()> {
    let sandbox = TestDirectory::create("single-format-acceptance")?;
    let root = sandbox.path();
    let (first, second) = (first_blob(), second_blob());
    let third = b"keep acceptance: the publication interrupted before HEAD. ".repeat(1_500);

    let initialized = FilesystemVersionTwoAdmission::initialize(root)?;
    assert_eq!(initialized.found(), VersionTwoInitializationPrefix::Empty);
    let mut writer = FilesystemVersionTwoCatalogPublisher::open(
        initialized.into_admission(),
        populated_store::policy()?,
    )?;
    let ingested = populated_store::publish_first(root, &mut writer, &[&first])?;
    assert_eq!(ingested.generation().get(), 1);
    drop(writer);

    drop(FilesystemVersionTwoAdmission::reopen(root)?);
    let layout = populated_store::identify(&first)?.record.id();
    let store = DurableStore::open(
        root,
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let mut output = Vec::new();
    let _receipt = store.reconstruct_layout(layout, &mut output)?;
    assert_eq!(output, first, "the first blob must reconstruct exactly");

    retain(root, b"acceptance-first", &first)?;
    let mut writer = open_publisher(root)?;
    let _second = publish(&mut writer, root, &second, Predecessor::Kept)??;
    drop(writer);
    retain(root, b"acceptance-second", &second)?;
    assert_eq!(current_generation(root)?, 2);

    let candidate = interrupted_publication(root, &third)?;
    assert_eq!(
        current_generation(root)?,
        2,
        "HEAD must not move before recovery"
    );
    recover(root, &candidate, 3)?;
    assert_reads(root, &first)?;
    assert_reads(root, &second)?;

    let before = witness(root)?;
    let mut writer = open_publisher(root)?;
    let outcome = publish(
        &mut writer,
        root,
        b"keep acceptance: a lossy candidate",
        Predecessor::Dropped,
    )?;
    drop(writer);
    let Err(RetainedCatalogPublicationError::Closure { .. }) = outcome else {
        return Err(format!("a candidate losing retained content published: {outcome:?}").into());
    };
    let mut after = witness(root)?;
    after.retain(|(path, _)| path != "staging/current.seg");
    assert_eq!(
        after, before,
        "the refused candidate must leave every byte in place"
    );
    assert_reads(root, &first)?;
    assert_reads(root, &second)
}

/// ADR-0010 portability: close, copy, make the original unavailable, reopen
/// and verify the copy, then interrupt a new write on the copy and recover it.
#[test]
fn a_backup_copy_reopens_verifies_and_recovers_an_interrupted_write() -> TestResult<()> {
    let first = first_blob();
    let original = durable_fixture::build("backup-original", &[&first])?;
    let backup = TestDirectory::create("backup-copy")?;
    copy_store(original.path(), backup.path())?;
    let moved = original.path().with_extension("unavailable");
    fs::rename(original.path(), &moved)?;
    let unavailable = FilesystemRetentionSnapshot::load(
        original.path(),
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
    );
    assert!(matches!(
        unavailable,
        Err(FilesystemRetentionSnapshotError::Admission { .. })
    ));

    let root = backup.path();
    drop(FilesystemVersionTwoAdmission::reopen(root)?);
    assert_reads(root, &first)?;
    let candidate = interrupted_publication(root, &second_blob())?;
    recover(root, &candidate, 2)?;
    retain(root, b"backup-second", &second_blob())?;
    assert_reads(root, &first)?;
    assert_reads(root, &second_blob())?;
    fs::remove_dir_all(moved)?;
    Ok(())
}
