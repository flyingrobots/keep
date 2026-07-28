//! One-pass bounded streaming ingestion into staged reference state.

use std::io::{ErrorKind, Read};

use crate::layout::check_entry_limit;
use crate::{AdmittedLayout, BlobHasher, BlobId, ChunkId, ChunkSpan, FastCdc, LayoutEntryLimit};

use super::chunk_staging::ReferenceChunkStaging;
use super::ingestion_error::IngestionAllocation;
use super::{IngestionError, ReferenceStore, StagedBlob};

macro_rules! read_buffer_bytes {
    () => {
        8_192
    };
}

const READ_BUFFER_BYTES: usize = read_buffer_bytes!();
// This bound makes more than one detector boundary per read impossible.
const _: () = assert!(read_buffer_bytes!() <= FastCdc::MINIMUM_CHUNK_LENGTH.get());

impl ReferenceStore {
    /// Reads one logical stream into invisible, validated staged work.
    ///
    /// The streaming engine retains one fixed 8 KiB read buffer, one buffer
    /// bounded by [`FastCdc::MAXIMUM_CHUNK_LENGTH`], detector/hash state, and
    /// layout metadata bounded by `entry_limit`. Layout admission and canonical
    /// identity calculation transiently materialize metadata proportional to
    /// that bounded entry count. The returned [`StagedBlob`] explicitly
    /// materializes new unique chunk bytes because this reference adapter is in
    /// memory, bounded by
    /// [`ReferenceStoreCapacity`](super::ReferenceStoreCapacity).
    ///
    /// This blocking operation performs caller-provided input I/O. Interrupted
    /// reads are retried. Empty and short reads are lawful; EOF is the only
    /// `Ok(0)` read.
    ///
    /// # Errors
    ///
    /// Returns [`IngestionError`] for source I/O, broken `Read` behavior,
    /// identity/chunking failure, bounded allocation or capacity refusal,
    /// conflicting existing bytes, layout admission, or canonical layout
    /// identity failure. No visible store state changes on failure.
    pub fn stage<R>(
        &self,
        source: &mut R,
        entry_limit: LayoutEntryLimit,
    ) -> Result<StagedBlob, IngestionError>
    where
        R: Read + ?Sized,
    {
        let mut staging = ReferenceChunkStaging::new(self);
        let (target, spans) = ingest_stream(source, &mut staging, entry_limit)?;
        let layout = AdmittedLayout::from_spans(
            target,
            crate::RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans,
            entry_limit,
        )
        .map_err(IngestionError::Layout)?;
        let layout_id = layout
            .encode_record()
            .map_err(IngestionError::LayoutEncoding)?
            .id();
        let (chunks, pending_bytes) = staging.into_parts();
        Ok(StagedBlob::new(layout, layout_id, chunks, pending_bytes))
    }

    /// Stages a stream only when its complete exact identity is `expected`.
    ///
    /// This performs the same bounded one-pass ingestion as
    /// [`ReferenceStore::stage`]. A mismatch returns both identities and drops
    /// all invisible staged bytes without mutating the store.
    ///
    /// # Errors
    ///
    /// Returns [`IngestionError::BlobIdentityMismatch`] when the complete
    /// staged stream names a different blob, or any failure documented by
    /// [`ReferenceStore::stage`].
    pub fn stage_expected<R>(
        &self,
        source: &mut R,
        expected: BlobId,
        entry_limit: LayoutEntryLimit,
    ) -> Result<StagedBlob, IngestionError>
    where
        R: Read + ?Sized,
    {
        let staged = self.stage(source, entry_limit)?;
        let observed = staged.target();
        if observed != expected {
            return Err(IngestionError::BlobIdentityMismatch { expected, observed });
        }
        Ok(staged)
    }
}

fn ingest_stream<R>(
    source: &mut R,
    staging: &mut ReferenceChunkStaging<'_>,
    entry_limit: LayoutEntryLimit,
) -> Result<(crate::BlobId, Vec<ChunkSpan>), IngestionError>
where
    R: Read + ?Sized,
{
    let maximum = usize::try_from(FastCdc::MAXIMUM_CHUNK_LENGTH.get()).map_err(|_source| {
        IngestionError::StreamLengthOverflow {
            accepted: 0,
            incoming: usize::MAX,
        }
    })?;
    let mut chunk_buffer = Vec::new();
    chunk_buffer
        .try_reserve_exact(maximum)
        .map_err(|source| IngestionError::Allocation {
            target: IngestionAllocation::ChunkBuffer,
            requested: maximum,
            source,
        })?;
    let mut state = StreamState::new(chunk_buffer, entry_limit);
    let mut read_buffer = [0_u8; READ_BUFFER_BYTES];
    loop {
        match source.read(&mut read_buffer) {
            Ok(0) => return state.finish(staging),
            Ok(observed) => {
                let bytes =
                    read_buffer
                        .get(..observed)
                        .ok_or(IngestionError::InvalidReadCount {
                            maximum: read_buffer.len(),
                            observed,
                        })?;
                state.accept(bytes, staging)?;
            }
            Err(source) if source.kind() == ErrorKind::Interrupted => {}
            Err(source) => return Err(IngestionError::Read { source }),
        }
    }
}

struct StreamState {
    detector: FastCdc,
    blob_hasher: BlobHasher,
    chunk_buffer: Vec<u8>,
    spans: Vec<ChunkSpan>,
    accepted: u64,
    entry_limit: LayoutEntryLimit,
}

impl StreamState {
    fn new(chunk_buffer: Vec<u8>, entry_limit: LayoutEntryLimit) -> Self {
        Self {
            detector: FastCdc::new(),
            blob_hasher: BlobHasher::new(),
            chunk_buffer,
            spans: Vec::new(),
            accepted: 0,
            entry_limit,
        }
    }

    fn accept(
        &mut self,
        bytes: &[u8],
        staging: &mut ReferenceChunkStaging<'_>,
    ) -> Result<(), IngestionError> {
        self.blob_hasher
            .update(bytes)
            .map_err(IngestionError::BlobHash)?;
        let mut emission = FeedEmission::None;
        self.detector
            .feed(bytes, |span| emission.record(span))
            .map_err(IngestionError::Chunking)?;
        let next_accepted = checked_accepted(self.accepted, bytes.len())?;
        match emission {
            FeedEmission::None => self.chunk_buffer.extend_from_slice(bytes),
            FeedEmission::One(span) => self.accept_boundary(bytes, span, staging)?,
            FeedEmission::Multiple => {
                return Err(IngestionError::MultipleBoundaries {
                    feed_length: bytes.len(),
                });
            }
        }
        self.accepted = next_accepted;
        Ok(())
    }

    fn accept_boundary(
        &mut self,
        bytes: &[u8],
        span: ChunkSpan,
        staging: &mut ReferenceChunkStaging<'_>,
    ) -> Result<(), IngestionError> {
        let local = boundary_index(self.accepted, span, bytes.len())?;
        let prefix = bytes
            .get(..local)
            .ok_or_else(|| IngestionError::BoundaryOutOfRange {
                feed_start: self.accepted,
                boundary: span.end().get(),
                feed_length: bytes.len(),
            })?;
        let remainder = bytes
            .get(local..)
            .ok_or_else(|| IngestionError::BoundaryOutOfRange {
                feed_start: self.accepted,
                boundary: span.end().get(),
                feed_length: bytes.len(),
            })?;
        prepare_span(&mut self.spans, self.entry_limit)?;
        self.chunk_buffer.extend_from_slice(prefix);
        stage_exact_chunk(staging, span, &self.chunk_buffer)?;
        self.spans.push(span);
        self.chunk_buffer.clear();
        self.chunk_buffer.extend_from_slice(remainder);
        Ok(())
    }

    fn finish(
        mut self,
        staging: &mut ReferenceChunkStaging<'_>,
    ) -> Result<(crate::BlobId, Vec<ChunkSpan>), IngestionError> {
        if let Some(span) = self.detector.finish().map_err(IngestionError::Chunking)? {
            prepare_span(&mut self.spans, self.entry_limit)?;
            stage_exact_chunk(staging, span, &self.chunk_buffer)?;
            self.spans.push(span);
        }
        Ok((self.blob_hasher.finish(), self.spans))
    }
}

enum FeedEmission {
    None,
    One(ChunkSpan),
    Multiple,
}

impl FeedEmission {
    const fn record(&mut self, span: ChunkSpan) {
        *self = match self {
            Self::None => Self::One(span),
            Self::One(_) | Self::Multiple => Self::Multiple,
        };
    }
}

fn checked_accepted(accepted: u64, incoming: usize) -> Result<u64, IngestionError> {
    let incoming_u64 = u64::try_from(incoming)
        .map_err(|_source| IngestionError::StreamLengthOverflow { accepted, incoming })?;
    accepted
        .checked_add(incoming_u64)
        .ok_or(IngestionError::StreamLengthOverflow { accepted, incoming })
}

fn boundary_index(
    feed_start: u64,
    span: ChunkSpan,
    feed_length: usize,
) -> Result<usize, IngestionError> {
    let boundary = span.end().get();
    let local = boundary
        .checked_sub(feed_start)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value <= feed_length)
        .ok_or(IngestionError::BoundaryOutOfRange {
            feed_start,
            boundary,
            feed_length,
        })?;
    Ok(local)
}

fn stage_exact_chunk(
    staging: &mut ReferenceChunkStaging<'_>,
    span: ChunkSpan,
    bytes: &[u8],
) -> Result<(), IngestionError> {
    let observed = ChunkId::hash_bytes(bytes).map_err(IngestionError::ChunkHash)?;
    if observed != span.id() {
        return Err(IngestionError::ChunkIdentityMismatch {
            expected: span.id(),
            observed,
        });
    }
    staging.stage_chunk(span.id(), bytes)
}

fn reserve_span(spans: &mut Vec<ChunkSpan>) -> Result<(), IngestionError> {
    spans
        .try_reserve(1)
        .map_err(|source| IngestionError::Allocation {
            target: IngestionAllocation::LayoutSpans,
            requested: 1,
            source,
        })
}

fn prepare_span(
    spans: &mut Vec<ChunkSpan>,
    entry_limit: LayoutEntryLimit,
) -> Result<(), IngestionError> {
    let observed = spans.len().checked_add(1).ok_or({
        IngestionError::Layout(crate::LayoutValidationError::EntryIndexOutOfRange {
            observed: usize::MAX,
        })
    })?;
    check_entry_limit(observed, entry_limit).map_err(IngestionError::Layout)?;
    reserve_span(spans)
}
