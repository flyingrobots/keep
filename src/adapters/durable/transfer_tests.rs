//! Copy laws across backends: a durable snapshot streams a committed
//! layout into a reference store, and a reference store streams into a
//! durable writer, each without buffering the blob and each verified end
//! to end.

use std::error::Error;
use std::io::Cursor;

use super::test_fixture::{durable_store, long_content};
use super::{DurableStore, DurableWriter};
use crate::adapters::pipeline::{
    NeverCancelled, TransferBounds, TransferWindow, WriteSink, transfer_layout,
};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store,
};
use crate::adapters::{FilesystemVersionTwoAdmission, ReaderAttemptLimit, copy_layout};
use crate::{LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity, StagingLimits};

#[test]
fn a_durable_snapshot_copies_into_a_reference_store_and_transfers_to_a_sink()
-> Result<(), Box<dyn Error>> {
    let long = long_content();
    let contents: [&[u8]; 2] = [&long, b"unanchored"];
    let (sandbox, published) = durable_store("durable-copy-out", &contents)?;
    let anchored = published.first().ok_or("anchored blob")?;
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?
    .snapshot()?;
    let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(4 * 1024 * 1024));
    let receipt = copy_layout(
        &snapshot,
        anchored.layout,
        &mut destination,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    assert_eq!(receipt.target(), anchored.target);
    let mut output = Vec::new();
    let _read = destination.reconstruct(anchored.target, &mut output)?;
    assert_eq!(output, long);
    let mut sink = WriteSink::new(Vec::new());
    let transfer = transfer_layout(
        &snapshot,
        anchored.layout,
        &mut sink,
        TransferBounds::new(TransferWindow::ONE, &NeverCancelled),
    )?;
    assert_eq!(sink.into_inner(), long);
    assert_eq!(transfer.read().receipt().target(), anchored.target);
    assert_eq!(transfer.acknowledgements(), transfer.segments());
    Ok(())
}

#[test]
fn a_reference_store_copies_into_a_durable_writer() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-copy-in")?;
    let long = long_content();
    let mut source = ReferenceStore::new(ReferenceStoreCapacity::new(4 * 1024 * 1024));
    let staged = source.stage(&mut Cursor::new(&long), LayoutEntryLimit::MAXIMUM)?;
    let blob = staged.commit(&mut source)?;
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?;
    let mut writer = DurableWriter::open(admission, sandbox.path(), catalog_policy()?)?;
    let receipt = copy_layout(
        &source,
        blob.layout_id(),
        &mut writer,
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    assert_eq!(receipt.target(), blob.target());
    assert_eq!(receipt.layout_id(), blob.layout_id());
    drop(writer);
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?
    .snapshot()?;
    let mut output = Vec::new();
    let _read = snapshot.reconstruct_layout(receipt.layout_id(), &mut output)?;
    assert_eq!(output, long);
    Ok(())
}
