//! This module owns the transfer: a windowed writer the read cores emit
//! into, handing each verified slice to the sink without a copy.

use std::error::Error;
use std::io::{self, Write};

use super::{
    CancellationSignal, TransferError, TransferReceipt, TransferSegment, TransferSink,
    TransferWindow,
};
use crate::{BlobId, ByteRange, ContentReads, LayoutId};

/// How a transfer is bounded: the acknowledgement window and the signal
/// consulted before every segment.
#[derive(Debug)]
pub struct TransferBounds<'signal, C: ?Sized> {
    window: TransferWindow,
    cancellation: &'signal C,
}

impl<C: ?Sized> Clone for TransferBounds<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: ?Sized> Copy for TransferBounds<'_, C> {}

impl<'signal, C: CancellationSignal + ?Sized> TransferBounds<'signal, C> {
    /// Acknowledge every `window` segments; stop once `cancellation` is
    /// raised.
    #[must_use]
    pub const fn new(window: TransferWindow, cancellation: &'signal C) -> Self {
        Self {
            window,
            cancellation,
        }
    }

    /// The acknowledgement window.
    #[must_use]
    pub const fn window(self) -> TransferWindow {
        self.window
    }
}

/// Transfers the exact committed layout `layout_id` from `view` to `sink`.
///
/// # Errors
///
/// Returns [`TransferError`] when the view refuses, the sink refuses, or
/// the signal is raised; partial output may have reached the sink.
pub fn transfer_layout<V, K, C>(
    view: &V,
    layout_id: LayoutId,
    sink: &mut K,
    bounds: TransferBounds<'_, C>,
) -> Result<TransferReceipt<V::ReconstructionReceipt>, TransferError<K::Error>>
where
    V: ContentReads + ?Sized,
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    run(sink, &bounds, |writer| {
        view.reconstruct_layout(layout_id, writer)
    })
}

/// Transfers `target` through the view's deterministic layout choice.
///
/// # Errors
///
/// As [`transfer_layout`].
pub fn transfer_blob<V, K, C>(
    view: &V,
    target: BlobId,
    sink: &mut K,
    bounds: TransferBounds<'_, C>,
) -> Result<TransferReceipt<V::ReconstructionReceipt>, TransferError<K::Error>>
where
    V: ContentReads + ?Sized,
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    run(sink, &bounds, |writer| view.reconstruct(target, writer))
}

/// Transfers exactly `requested` of `target`, authenticating only the
/// overlapping chunks.
///
/// # Errors
///
/// As [`transfer_layout`].
pub fn transfer_range<V, K, C>(
    view: &V,
    target: BlobId,
    requested: ByteRange,
    sink: &mut K,
    bounds: TransferBounds<'_, C>,
) -> Result<TransferReceipt<V::RangeReceipt>, TransferError<K::Error>>
where
    V: ContentReads + ?Sized,
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    run(sink, &bounds, |writer| {
        view.read_range(target, requested, writer)
    })
}

/// Transfers exactly `requested` through the exact committed layout.
///
/// # Errors
///
/// As [`transfer_layout`].
pub fn transfer_layout_range<V, K, C>(
    view: &V,
    layout_id: LayoutId,
    requested: ByteRange,
    sink: &mut K,
    bounds: TransferBounds<'_, C>,
) -> Result<TransferReceipt<V::RangeReceipt>, TransferError<K::Error>>
where
    V: ContentReads + ?Sized,
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    run(sink, &bounds, |writer| {
        view.read_layout_range(layout_id, requested, writer)
    })
}

fn run<K, C, R, E>(
    sink: &mut K,
    bounds: &TransferBounds<'_, C>,
    read: impl FnOnce(&mut WindowedWriter<'_, K, C>) -> Result<R, E>,
) -> Result<TransferReceipt<R>, TransferError<K::Error>>
where
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
    R: Copy,
    E: Error + 'static,
{
    let window = bounds.window;
    let mut writer = WindowedWriter {
        sink,
        window,
        cancellation: bounds.cancellation,
        segments: 0,
        bytes: 0,
        in_window: 0,
        acknowledgements: 0,
        failure: None,
    };
    let receipt = match read(&mut writer) {
        Ok(receipt) => receipt,
        Err(source) => {
            return Err(match writer.failure.take() {
                Some(Failure::Sink(error)) => TransferError::Sink(error),
                Some(Failure::Cancelled) => TransferError::Cancelled {
                    segments: writer.segments,
                    bytes: writer.bytes,
                },
                Some(Failure::Accounting) => TransferError::Accounting,
                None => TransferError::Read(Box::new(source)),
            });
        }
    };
    if writer.in_window > 0 {
        writer.acknowledge().map_err(TransferError::Sink)?;
    }
    writer.sink.complete().map_err(TransferError::Sink)?;
    Ok(TransferReceipt::new(
        receipt,
        writer.segments,
        writer.bytes,
        writer.acknowledgements,
        window,
    ))
}

enum Failure<E> {
    Sink(E),
    Cancelled,
    Accounting,
}

#[derive(Clone, Copy, Debug)]
enum TransferStop {
    Sink,
    Cancelled,
    Accounting,
}

impl std::fmt::Display for TransferStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "transfer stopped: {self:?}")
    }
}

impl std::error::Error for TransferStop {}

/// The writer the read cores emit into. Each `write` is one verified slice
/// of an immutable chunk, handed to the sink as a segment without a copy.
struct WindowedWriter<'sink, K: TransferSink + ?Sized, C: ?Sized> {
    sink: &'sink mut K,
    window: TransferWindow,
    cancellation: &'sink C,
    segments: u64,
    bytes: u64,
    in_window: u32,
    acknowledgements: u64,
    failure: Option<Failure<K::Error>>,
}

impl<K, C> WindowedWriter<'_, K, C>
where
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    fn acknowledge(&mut self) -> Result<(), K::Error> {
        self.sink.acknowledge()?;
        self.in_window = 0;
        self.acknowledgements = self.acknowledgements.saturating_add(1);
        Ok(())
    }

    fn fail(&mut self, failure: Failure<K::Error>) -> io::Error {
        let reason = match &failure {
            Failure::Sink(_) => TransferStop::Sink,
            Failure::Cancelled => TransferStop::Cancelled,
            Failure::Accounting => TransferStop::Accounting,
        };
        self.failure = Some(failure);
        io::Error::other(reason)
    }

    fn apply(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(self.fail(Failure::Cancelled));
        }
        let segment = TransferSegment::new(self.segments, self.bytes, bytes);
        if let Err(error) = self.sink.apply(segment) {
            return Err(self.fail(Failure::Sink(error)));
        }
        let Some(next_bytes) = u64::try_from(bytes.len())
            .ok()
            .and_then(|length| self.bytes.checked_add(length))
        else {
            return Err(self.fail(Failure::Accounting));
        };
        let Some(next_segments) = self.segments.checked_add(1) else {
            return Err(self.fail(Failure::Accounting));
        };
        self.bytes = next_bytes;
        self.segments = next_segments;
        self.in_window = self.in_window.saturating_add(1);
        if self.in_window >= self.window.get()
            && let Err(error) = self.acknowledge()
        {
            return Err(self.fail(Failure::Sink(error)));
        }
        Ok(())
    }
}

impl<K, C> Write for WindowedWriter<'_, K, C>
where
    K: TransferSink + ?Sized,
    C: CancellationSignal + ?Sized,
{
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        self.apply(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
