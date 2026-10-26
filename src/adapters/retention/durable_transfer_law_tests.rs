//! This module owns transfer-pipeline laws with a durable snapshot as the
//! source, against the independent one-zero corpus.
//!
//! Size: medium (owned filesystem scratch, no network or sleeps).
//! Oracle: the frozen one-zero bundle and root identify the single byte [0].
//! Delete when durable snapshots stop implementing `TransferSource` or
//! backend-neutral transfer laws subsume these outcomes.

use std::error::Error;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy, CopyError,
    DurableSnapshot, LayoutEntryLimit, NeverCancelled, ReaderAttemptLimit, ReferenceStore,
    ReferenceStoreCapacity, RetentionAnchor, SegmentReadPolicy, SegmentRecordLimit,
    StagingLimits, TransferBounds, TransferSourceError, TransferWindow, WriteSink, copy_layout,
    execute_retention_publication, transfer_layout,
};

fn published_snapshot(
    name: &str,
) -> Result<(TestDirectory, DurableSnapshot, RetentionAnchor), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = *root
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let snapshot = DurableSnapshot::open(
        sandbox.path(),
        CatalogRestartPolicy::new(
            SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    )?;
    Ok((sandbox, snapshot, anchor))
}

#[test]
fn a_durable_snapshot_copies_its_exact_layout_into_a_reference_store()
-> Result<(), Box<dyn Error>> {
    let (_sandbox, snapshot, anchor) = published_snapshot("durable-transfer-copy")?;
    let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let receipt = copy_layout(
        &snapshot,
        anchor.layout_id(),
        &mut destination,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    assert_eq!(receipt.target(), anchor.blob_id());
    assert_eq!(receipt.layout_id(), anchor.layout_id());
    let mut output = Vec::new();
    let _read = destination.reconstruct_layout(anchor.layout_id(), &mut output)?;
    assert_eq!(output, [0]);
    Ok(())
}

#[test]
fn a_durable_snapshot_transfers_authenticated_bytes_into_a_sink() -> Result<(), Box<dyn Error>> {
    let (_sandbox, snapshot, anchor) = published_snapshot("durable-transfer-sink")?;
    let mut sink = WriteSink::new(Vec::new());
    let window = TransferWindow::new(1).ok_or("window")?;
    let receipt = transfer_layout(
        &snapshot,
        anchor.layout_id(),
        &mut sink,
        TransferBounds::new(window, &NeverCancelled),
    )?;
    assert_eq!(receipt.read().receipt().target(), anchor.blob_id());
    assert_eq!(receipt.bytes(), 1);
    assert_eq!(receipt.acknowledgements(), receipt.segments());
    assert_eq!(sink.into_inner(), [0]);
    Ok(())
}

#[test]
fn a_missing_durable_layout_refuses_the_copy_before_staging() -> Result<(), Box<dyn Error>> {
    let (_sandbox, snapshot, anchor) = published_snapshot("durable-transfer-missing")?;
    let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let absent = destination
        .stage(&mut std::io::Cursor::new(b"absent"), LayoutEntryLimit::MAXIMUM)?
        .layout_id();
    let refusal = copy_layout(
        &snapshot,
        absent,
        &mut destination,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )
    .err()
    .ok_or("a missing layout was copied")?;
    assert!(
        matches!(&refusal, CopyError::Source(source)
            if matches!(source.as_ref(), TransferSourceError::LayoutMissing { requested }
                if *requested == absent)),
        "unexpected refusal: {refusal:?}"
    );
    assert!(!destination.contains_blob(anchor.blob_id()));
    Ok(())
}
