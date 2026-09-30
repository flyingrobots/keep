//! Transfer pipeline against a caller-owned copy loop.

use std::io::Cursor;

use divan::counter::BytesCount;
use divan::{Bencher, black_box};
use keep::{
    LayoutEntryLimit, NeverCancelled, PublishedBlob, ReferenceStore, ReferenceStoreCapacity,
    StagingLimits, TransferBounds, TransferWindow, WriteSink, copy_layout, transfer_layout,
};

const REPRESENTATIVE_INPUT_BYTES: usize = 1_048_576;
const LARGE_INPUT_BYTES: usize = 4_194_304;
const CAPACITY: usize = 64 * 1024 * 1024;

fn main() {
    divan::main();
}

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

fn published(length: usize) -> (ReferenceStore, PublishedBlob) {
    let bytes = deterministic_bytes(length);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let staged = store
        .stage(&mut Cursor::new(&bytes), LayoutEntryLimit::MAXIMUM)
        .expect("stage");
    let blob = staged.commit(&mut store).expect("commit");
    (store, blob)
}

/// Read-to-write through the pipeline: each verified chunk slice reaches
/// the sink without an intermediate buffer.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn read_to_write_pipeline(bencher: Bencher<'_, '_>, length: usize) {
    let (store, blob) = published(length);
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut sink = WriteSink::new(std::io::sink());
        let receipt = transfer_layout(
            black_box(&store),
            blob.layout_id(),
            &mut sink,
            TransferBounds::new(TransferWindow::ONE, &NeverCancelled),
        )
        .expect("transfer");
        black_box(receipt.bytes())
    });
}

/// Read-to-write the way a caller does it today: reconstruct into an owned
/// buffer, then write the buffer out.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn read_to_write_copy_loop(bencher: Bencher<'_, '_>, length: usize) {
    let (store, blob) = published(length);
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut buffer = Vec::new();
        let _receipt = store
            .reconstruct_layout(blob.layout_id(), &mut buffer)
            .expect("reconstruct");
        std::io::copy(&mut Cursor::new(&buffer), &mut std::io::sink()).expect("copy");
        black_box(buffer.len())
    });
}

/// Copy-to-write through the pipeline: the destination stages the source's
/// verified chunks directly.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn copy_to_write_pipeline(bencher: Bencher<'_, '_>, length: usize) {
    let (store, blob) = published(length);
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
        let receipt = copy_layout(
            black_box(&store),
            blob.layout_id(),
            &mut destination,
            StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
        )
        .expect("copy");
        black_box(receipt.target())
    });
}

/// Copy-to-write the way a caller does it today: reconstruct into an owned
/// buffer, then stage the buffer.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn copy_to_write_copy_loop(bencher: Bencher<'_, '_>, length: usize) {
    let (store, blob) = published(length);
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
        let mut buffer = Vec::new();
        let _receipt = store
            .reconstruct_layout(blob.layout_id(), &mut buffer)
            .expect("reconstruct");
        let staged = destination
            .stage(&mut Cursor::new(&buffer), LayoutEntryLimit::MAXIMUM)
            .expect("stage");
        let published = staged.commit(&mut destination).expect("commit");
        black_box(published.target())
    });
}
