//! Transfer pipeline against a caller-owned copy loop.

use std::error::Error;
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

/// Refuses to benchmark at all when either input cannot be published, so
/// a benchmark function never has to report a setup failure itself.
fn main() -> Result<(), Box<dyn Error>> {
    for length in [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES] {
        let _published = published(length)?;
    }
    divan::main();
    Ok(())
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

type Setup = Result<(ReferenceStore, PublishedBlob), Box<dyn Error>>;

fn published(length: usize) -> Setup {
    let bytes = deterministic_bytes(length);
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let staged = store.stage(&mut Cursor::new(&bytes), LayoutEntryLimit::MAXIMUM)?;
    let blob = staged.commit(&mut store)?;
    Ok((store, blob))
}

/// Publishes the input; `main` already proved this succeeds for every
/// benchmarked length, so a bench function only skips on a refusal it
/// cannot report.
fn setup(length: usize) -> Option<(ReferenceStore, PublishedBlob)> {
    published(length).ok()
}

/// Read-to-write through the pipeline: each verified chunk slice reaches
/// the sink without an intermediate buffer.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn read_to_write_pipeline(bencher: Bencher<'_, '_>, length: usize) {
    let Some((store, blob)) = setup(length) else {
        return;
    };
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut sink = WriteSink::new(std::io::sink());
        transfer_layout(
            black_box(&store),
            blob.layout_id(),
            &mut sink,
            TransferBounds::new(TransferWindow::ONE, &NeverCancelled),
        )
        .map(|receipt| black_box(receipt.bytes()))
    });
}

/// Read-to-write the way a caller does it today: reconstruct into an owned
/// buffer, then write the buffer out.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn read_to_write_copy_loop(bencher: Bencher<'_, '_>, length: usize) {
    let Some((store, blob)) = setup(length) else {
        return;
    };
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut buffer = Vec::new();
        store
            .reconstruct_layout(blob.layout_id(), &mut buffer)
            .map_err(|error| error.to_string())
            .and_then(|_receipt| {
                std::io::copy(&mut Cursor::new(&buffer), &mut std::io::sink())
                    .map_err(|error| error.to_string())
            })
            .map(|_copied| black_box(buffer.len()))
    });
}

/// Copy-to-write through the pipeline: the destination stages the source's
/// verified chunks directly.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn copy_to_write_pipeline(bencher: Bencher<'_, '_>, length: usize) {
    let Some((store, blob)) = setup(length) else {
        return;
    };
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
        copy_layout(
            black_box(&store),
            blob.layout_id(),
            &mut destination,
            StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
        )
        .map(|receipt| black_box(receipt.target()))
    });
}

/// Copy-to-write the way a caller does it today: reconstruct into an owned
/// buffer, then stage the buffer.
#[divan::bench(args = [REPRESENTATIVE_INPUT_BYTES, LARGE_INPUT_BYTES])]
fn copy_to_write_copy_loop(bencher: Bencher<'_, '_>, length: usize) {
    let Some((store, blob)) = setup(length) else {
        return;
    };
    bencher.counter(BytesCount::new(length)).bench_local(|| {
        let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
        let mut buffer = Vec::new();
        store
            .reconstruct_layout(blob.layout_id(), &mut buffer)
            .map_err(|error| error.to_string())
            .and_then(|_receipt| {
                destination
                    .stage(&mut Cursor::new(&buffer), LayoutEntryLimit::MAXIMUM)
                    .map_err(|error| error.to_string())
            })
            .and_then(|staged| {
                staged
                    .commit(&mut destination)
                    .map_err(|error| error.to_string())
            })
            .map(|published| black_box(published.target()))
    });
}
