//! Transfer pipeline memory laws, measured against the library as shipped:
//! read-to-write allocates nothing beyond the sink, and copy-to-write
//! allocates less than a caller-owned copy loop.

use std::error::Error;
use std::io::Cursor;

use allocation_counter::{AllocationInfo, measure};
use keep::{
    LayoutEntryLimit, NeverCancelled, PublishedBlob, ReferenceStore, ReferenceStoreCapacity,
    StagingLimits, TransferBounds, TransferWindow, WriteSink, copy_layout, transfer_layout,
};

const CAPACITY: usize = 16 * 1024 * 1024;
const INPUT_BYTES: usize = 1_048_576;

fn deterministic_bytes(length: usize) -> Vec<u8> {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).unwrap_or_default()
        })
        .collect()
}

fn published(bytes: &[u8]) -> Result<(ReferenceStore, PublishedBlob), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let staged = store.stage(&mut Cursor::new(bytes), LayoutEntryLimit::MAXIMUM)?;
    let blob = staged.commit(&mut store)?;
    Ok((store, blob))
}

#[test]
fn read_to_write_allocates_nothing_beyond_the_sink() -> Result<(), Box<dyn Error>> {
    let bytes = deterministic_bytes(INPUT_BYTES);
    let (store, blob) = published(&bytes)?;
    let mut sink = WriteSink::new(Vec::with_capacity(bytes.len()));
    let mut outcome = None;
    let observed = measure(|| {
        outcome = Some(transfer_layout(
            &store,
            blob.layout_id(),
            &mut sink,
            TransferBounds::new(TransferWindow::ONE, &NeverCancelled),
        ));
    });
    let receipt = outcome.ok_or("no outcome")??;
    assert_eq!(observed, AllocationInfo::default());
    assert_eq!(receipt.bytes(), u64::try_from(bytes.len())?);
    assert_eq!(sink.into_inner(), bytes);
    Ok(())
}

#[test]
fn copy_to_write_allocates_less_than_a_caller_owned_copy_loop() -> Result<(), Box<dyn Error>> {
    let bytes = deterministic_bytes(INPUT_BYTES);
    let (source, blob) = published(&bytes)?;
    let limits = StagingLimits::entries(LayoutEntryLimit::MAXIMUM);

    let mut piped = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let mut copy = None;
    let pipeline = measure(|| {
        copy = Some(copy_layout(&source, blob.layout_id(), &mut piped, limits));
    });
    let receipt = copy.ok_or("no outcome")??;
    assert_eq!(receipt.target(), blob.target());

    let mut looped = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let mut loop_outcome = None;
    let copy_loop = measure(|| {
        loop_outcome = Some(caller_owned_copy_loop(&source, blob, &mut looped, limits));
    });
    loop_outcome.ok_or("no outcome")??;

    assert!(
        pipeline.bytes_total < copy_loop.bytes_total && pipeline.bytes_max < copy_loop.bytes_max,
        "pipeline {pipeline:?}, copy loop {copy_loop:?}"
    );
    let mut output = Vec::new();
    let _read = piped.reconstruct(receipt.target(), &mut output)?;
    assert_eq!(output, bytes);
    Ok(())
}

fn caller_owned_copy_loop(
    source: &ReferenceStore,
    blob: PublishedBlob,
    destination: &mut ReferenceStore,
    limits: StagingLimits,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let _read = source.reconstruct_layout(blob.layout_id(), &mut buffer)?;
    let staged = destination.stage_bounded(&mut Cursor::new(&buffer), limits)?;
    let _receipt = staged.commit(destination)?;
    Ok(())
}
