//! This module owns pinned-read lifetime and exact durable admission failure laws.
//!
//! Size: medium; owned filesystem scratch, deterministic publication order and kernel try-lock.
//! Oracle: published generations, the independent one-zero corpus, and typed I/O boundaries.
//! Delete when durable snapshots are removed or stronger public laws subsume these outcomes.

use std::error::Error;
use std::fs;

use rustix::fs::{FlockOperation, flock};
use rustix::io::Errno;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, SEGMENT_NAME, fixture, initial_preparation, open_authority, successor_preparation,
};
use crate::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionRoot,
    CatalogRestartByteLimit, CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy,
    DurableSnapshot, DurableStoreError, FilesystemRetentionPublicationAuthority,
    FilesystemRetentionSnapshotError, FilesystemVersionTwoAdmission, LayoutEntryLimit,
    ReaderAttemptLimit, RetentionPolicy, RetentionRoot, SegmentReadPolicy, SegmentRecordLimit,
    execute_retention_publication,
};

fn snapshot(path: &std::path::Path) -> Result<DurableSnapshot, Box<dyn Error>> {
    Ok(DurableSnapshot::open(
        path,
        CatalogRestartPolicy::new(
            SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    )?)
}

#[test]
fn a_pinned_read_keeps_its_retained_view_after_release_publication() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-pinned-release")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let current = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = current
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let pinned = snapshot(sandbox.path())?;
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;
    let mut authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    let state = authority.observe_current()?.ok_or("retention absent")?;
    let manifest = AdmittedRetentionManifest::decode(state.manifest_bytes())?;
    let successor = RetentionRoot::new(
        current.root().namespace().clone(),
        current.root().generation().successor()?,
        RetentionPolicy::new(current.root().profile(), current.root().limits()),
        Some(current.digest()),
        Vec::new(),
    )?;
    let encoded = CanonicalRetentionRoot::from_root(&successor)?;
    let release = successor_preparation(&current, &manifest, encoded.encoded())?;
    let _released = execute_retention_publication(&mut authority, &release)?;
    drop(authority);
    let mut output = Vec::new();
    let receipt = pinned.reconstruct(anchor.blob_id(), &mut output)?;
    assert_eq!(
        output,
        [0],
        "old view must still emit its exact retained bytes"
    );
    assert_eq!(
        receipt
            .view()
            .retention()
            .ok_or("old head absent")?
            .generation()
            .get(),
        1
    );
    let fresh = snapshot(sandbox.path())?;
    assert!(
        !fresh.contains_blob(anchor.blob_id())?,
        "fresh view must observe release"
    );
    assert_eq!(
        fresh
            .view()
            .retention()
            .ok_or("new head absent")?
            .generation()
            .get(),
        2
    );
    Ok(())
}

#[test]
fn durable_snapshot_holds_the_collector_fence_until_drop() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("durable-collector-fence")?;
    drop(authority);
    let pinned = snapshot(sandbox.path())?;
    let collector = fs::File::open(sandbox.path().join("reader.lock"))?;
    assert_eq!(
        flock(&collector, FlockOperation::NonBlockingLockExclusive),
        Err(Errno::WOULDBLOCK)
    );
    drop(pinned);
    flock(&collector, FlockOperation::NonBlockingLockExclusive)?;
    Ok(())
}

#[test]
fn an_unreadable_segment_preserves_the_operational_open_failure() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("durable-missing-segment")?;
    drop(authority);
    fs::remove_file(sandbox.path().join("segments").join(SEGMENT_NAME))?;
    let failure = snapshot(sandbox.path())
        .err()
        .ok_or("missing segment was admitted")?;
    assert!(
        matches!(failure.downcast_ref::<DurableStoreError>(), Some(DurableStoreError::Snapshot(source))
        if matches!(source.as_ref(), FilesystemRetentionSnapshotError::Catalog { source }
            if matches!(source, CatalogRestartError::Io { phase: CatalogRestartPhase::OpenSegment, source }
                if source.kind() == std::io::ErrorKind::NotFound))),
        "missing physical evidence must preserve OpenSegment/NotFound: {failure:?}"
    );
    Ok(())
}
