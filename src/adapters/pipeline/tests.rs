//! Pipeline laws over the reference backend: read-to-write transfers every
//! verified slice as the read core's own borrowed bytes; ranges transfer
//! exactly;
//! a window of one acknowledges every segment; a sink failure and a
//! cancellation each yield no receipt; a write sink applies segments
//! exactly once in order; copy-to-write moves a blob between stores while
//! allocating less than a caller-owned copy loop.

use std::error::Error;
use std::io::{self, Cursor, Write};

use allocation_counter::measure;

use super::{
    CancellationFlag, CopyError, NeverCancelled, TransferBounds, TransferError, TransferSegment,
    TransferSink, TransferSourceError, TransferWindow, WriteSink, WriteSinkError, copy_layout,
    transfer_layout, transfer_layout_range, transfer_range,
};
use crate::{
    ByteLength, ByteOffset, ByteRange, FastCdc, LayoutEntryLimit, PublishedBlob, ReferenceStore,
    ReferenceStoreCapacity, StagingLimits,
};

const CAPACITY: usize = 8 * 1024 * 1024;

fn content(length: usize) -> Vec<u8> {
    let mut state = 0x0f1e_2d3c_4b5a_6978_u64;
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
    let receipt = staged.commit(&mut store)?;
    Ok((store, receipt))
}

fn span_count(bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
    let mut detector = FastCdc::new();
    let mut count = 0_usize;
    detector.feed(bytes, |_span| count = count.saturating_add(1))?;
    if detector.finish()?.is_some() {
        count = count.saturating_add(1);
    }
    Ok(count)
}

fn window(segments: u32) -> Result<TransferWindow, Box<dyn Error>> {
    TransferWindow::new(segments).ok_or_else(|| "zero window".into())
}

#[test]
fn read_to_write_transfers_every_verified_slice_in_order() -> Result<(), Box<dyn Error>> {
    let bytes = content(400 * 1024);
    let (store, blob) = published(&bytes)?;
    let entries = u64::try_from(span_count(&bytes)?)?;
    let mut sink = WriteSink::new(Vec::with_capacity(bytes.len()));
    let receipt = transfer_layout(
        &store,
        blob.layout_id(),
        &mut sink,
        TransferBounds::new(window(3)?, &NeverCancelled),
    )?;
    assert_eq!(sink.into_inner(), bytes);
    assert_eq!(receipt.segments(), entries);
    assert_eq!(receipt.bytes(), u64::try_from(bytes.len())?);
    assert_eq!(receipt.acknowledgements(), entries.div_ceil(3));
    assert_eq!(receipt.read().target(), blob.target());
    Ok(())
}

#[test]
fn ranges_transfer_exactly_the_requested_bytes() -> Result<(), Box<dyn Error>> {
    let bytes = content(300 * 1024);
    let (store, blob) = published(&bytes)?;
    let requested = ByteRange::new(ByteOffset::new(70_000), ByteLength::new(150_000))?;
    let mut sink = WriteSink::new(Vec::new());
    let receipt = transfer_range(
        &store,
        blob.target(),
        requested,
        &mut sink,
        TransferBounds::new(window(2)?, &NeverCancelled),
    )?;
    assert_eq!(
        Some(sink.into_inner().as_slice()),
        bytes.get(70_000..220_000)
    );
    assert_eq!(receipt.bytes(), 150_000);
    let mut exact = WriteSink::new(Vec::new());
    let _receipt = transfer_layout_range(
        &store,
        blob.layout_id(),
        requested,
        &mut exact,
        TransferBounds::new(window(2)?, &NeverCancelled),
    )?;
    assert_eq!(
        Some(exact.into_inner().as_slice()),
        bytes.get(70_000..220_000)
    );
    Ok(())
}

#[test]
fn a_window_of_one_acknowledges_every_segment() -> Result<(), Box<dyn Error>> {
    let bytes = content(300 * 1024);
    let (store, blob) = published(&bytes)?;
    let mut sink = WriteSink::new(Vec::new());
    let receipt = transfer_layout(
        &store,
        blob.layout_id(),
        &mut sink,
        TransferBounds::new(TransferWindow::ONE, &NeverCancelled),
    )?;
    assert_eq!(receipt.acknowledgements(), receipt.segments());
    assert!(receipt.segments() > 1);
    Ok(())
}

struct FailAfter {
    written: usize,
    after: usize,
}

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.written >= self.after {
            return Err(io::Error::other("the sink went away"));
        }
        self.written = self.written.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_sink_that_fails_mid_window_yields_no_receipt() -> Result<(), Box<dyn Error>> {
    let bytes = content(300 * 1024);
    let (store, blob) = published(&bytes)?;
    let mut sink = WriteSink::new(FailAfter {
        written: 0,
        after: 100_000,
    });
    let refusal = transfer_layout(
        &store,
        blob.layout_id(),
        &mut sink,
        TransferBounds::new(window(4)?, &NeverCancelled),
    );
    assert!(matches!(
        refusal,
        Err(TransferError::Sink(WriteSinkError::Write { index, .. })) if index >= 1
    ));
    assert!(sink.applied() >= 1);
    Ok(())
}

struct CancelAfter {
    inner: WriteSink<Vec<u8>>,
    flag: CancellationFlag,
    after: u64,
}

impl TransferSink for CancelAfter {
    type Error = WriteSinkError;

    fn apply(&mut self, segment: TransferSegment<'_>) -> Result<(), WriteSinkError> {
        self.inner.apply(segment)?;
        if self.inner.applied() >= self.after {
            self.flag.cancel();
        }
        Ok(())
    }

    fn acknowledge(&mut self) -> Result<(), WriteSinkError> {
        self.inner.acknowledge()
    }

    fn complete(&mut self) -> Result<(), WriteSinkError> {
        self.inner.complete()
    }
}

#[test]
fn cancellation_stops_before_the_next_segment_and_never_becomes_success()
-> Result<(), Box<dyn Error>> {
    let bytes = content(300 * 1024);
    let (store, blob) = published(&bytes)?;
    let flag = CancellationFlag::new();
    let mut sink = CancelAfter {
        inner: WriteSink::new(Vec::new()),
        flag: flag.clone(),
        after: 2,
    };
    let refusal = transfer_layout(
        &store,
        blob.layout_id(),
        &mut sink,
        TransferBounds::new(window(8)?, &flag),
    );
    let Err(TransferError::Cancelled {
        segments,
        bytes: applied,
    }) = refusal
    else {
        return Err("expected cancellation".into());
    };
    assert_eq!(segments, 2);
    let partial = sink.inner.into_inner();
    assert_eq!(u64::try_from(partial.len())?, applied);
    assert!(partial.len() < bytes.len());
    assert_eq!(Some(partial.as_slice()), bytes.get(..partial.len()));
    Ok(())
}

#[test]
fn a_write_sink_applies_segments_exactly_once_in_order() {
    let mut sink = WriteSink::new(Vec::new());
    let first = TransferSegment::new(0, 0, b"abc");
    assert!(sink.apply(first).is_ok());
    assert!(matches!(
        sink.apply(first),
        Err(WriteSinkError::OutOfOrder {
            expected_index: 1,
            observed_index: 0,
            ..
        })
    ));
    assert!(matches!(
        sink.apply(TransferSegment::new(1, 0, b"d")),
        Err(WriteSinkError::OutOfOrder {
            expected_offset: 3,
            observed_offset: 0,
            ..
        })
    ));
    assert!(sink.apply(TransferSegment::new(1, 3, b"d")).is_ok());
    assert_eq!(sink.into_inner(), b"abcd");
}

#[test]
fn copy_to_write_moves_a_blob_between_stores_allocating_less_than_a_copy_loop()
-> Result<(), Box<dyn Error>> {
    let bytes = content(600 * 1024);
    let (source, blob) = published(&bytes)?;
    let limits = StagingLimits::entries(LayoutEntryLimit::MAXIMUM);
    let mut piped = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let mut copy = None;
    let pipeline = measure(|| {
        copy = Some(copy_layout(&source, blob.layout_id(), &mut piped, limits));
    });
    let receipt = copy.ok_or("no outcome")??;
    assert_eq!(receipt.target(), blob.target());
    let mut output = Vec::new();
    let _read = piped.reconstruct(receipt.target(), &mut output)?;
    assert_eq!(output, bytes);

    let mut looped = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let mut loop_outcome = None;
    let copy_loop = measure(|| {
        loop_outcome = Some(caller_owned_copy_loop(&source, blob, &mut looped, limits));
    });
    loop_outcome.ok_or("no outcome")??;
    assert!(
        pipeline.bytes_total < copy_loop.bytes_total,
        "pipeline {} bytes, copy loop {} bytes",
        pipeline.bytes_total,
        copy_loop.bytes_total
    );
    Ok(())
}

/// What a caller does today: reconstruct into an owned buffer, then stage
/// that buffer into the destination.
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

#[test]
fn a_copy_of_an_absent_layout_and_a_wrong_identity_commit_nothing() -> Result<(), Box<dyn Error>> {
    let bytes = content(300 * 1024);
    let (source, blob) = published(&bytes)?;
    let mut destination = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let limits = StagingLimits::entries(LayoutEntryLimit::MAXIMUM);
    let absent = ReferenceStore::new(ReferenceStoreCapacity::new(CAPACITY));
    let refusal = copy_layout(&absent, blob.layout_id(), &mut destination, limits);
    let Err(CopyError::Source(refused)) = refusal else {
        return Err("expected a source refusal".into());
    };
    assert!(matches!(
        *refused,
        TransferSourceError::LayoutMissing { requested } if requested == blob.layout_id()
    ));
    let tight = StagingLimits::entries(LayoutEntryLimit::new(1)?);
    let refusal = copy_layout(&source, blob.layout_id(), &mut destination, tight);
    assert!(
        matches!(refusal, Err(CopyError::Stage(_))),
        "expected a staging refusal, got {refusal:?}"
    );
    assert!(!destination.contains_blob(blob.target()));
    Ok(())
}
