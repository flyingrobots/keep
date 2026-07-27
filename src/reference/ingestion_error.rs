//! Typed failures while staging a bounded logical stream.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;
use std::io;

use crate::{
    BlobHashError, BlobId, ChunkHashError, ChunkId, ChunkingError, LayoutEncodeError,
    LayoutValidationError,
};

/// Allocation whose explicit bounded reservation failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestionAllocation {
    /// Fixed maximum-size current-chunk buffer.
    ChunkBuffer,
    /// One owned chunk admitted by the reference adapter.
    StagedChunk,
    /// Ordered detector spans used to admit the layout.
    LayoutSpans,
}

/// Failure while staging a stream into the non-durable reference adapter.
#[derive(Debug)]
pub enum IngestionError {
    /// The logical byte source failed.
    Read {
        /// Original source error.
        source: io::Error,
    },
    /// A broken reader reported more bytes than the supplied buffer.
    InvalidReadCount {
        /// Supplied buffer capacity.
        maximum: usize,
        /// Count reported by the reader.
        observed: usize,
    },
    /// Logical blob identity accounting failed.
    BlobHash(BlobHashError),
    /// Staged exact bytes do not match a caller-supplied logical identity.
    BlobIdentityMismatch {
        /// Logical identity required by the caller.
        expected: BlobId,
        /// Logical identity calculated from the complete staged stream.
        observed: BlobId,
    },
    /// Registered CDC processing failed.
    Chunking(ChunkingError),
    /// Independent staged-chunk verification failed.
    ChunkHash(ChunkHashError),
    /// Detector output did not name the buffered exact bytes.
    ChunkIdentityMismatch {
        /// Identity emitted by the detector.
        expected: ChunkId,
        /// Identity calculated from the staged bytes.
        observed: ChunkId,
    },
    /// A detector boundary escaped the current feed slice.
    BoundaryOutOfRange {
        /// Absolute feed start.
        feed_start: u64,
        /// Absolute emitted boundary.
        boundary: u64,
        /// Current feed byte count.
        feed_length: usize,
    },
    /// More than one boundary appeared in one fixed sub-minimum read.
    MultipleBoundaries {
        /// Current feed byte count.
        feed_length: usize,
    },
    /// Checked accepted-byte accounting overflowed.
    StreamLengthOverflow {
        /// Bytes accepted before the current read.
        accepted: u64,
        /// Current read byte count.
        incoming: usize,
    },
    /// A bounded allocation could not be reserved.
    Allocation {
        /// Allocation purpose.
        target: IngestionAllocation,
        /// Requested byte or element count.
        requested: usize,
        /// Original allocation error.
        source: TryReserveError,
    },
    /// New unique chunk bytes exceed the reference-store capacity.
    CapacityExceeded {
        /// Configured materialized-byte capacity.
        capacity: usize,
        /// Materialized bytes required by the staged state.
        attempted: usize,
    },
    /// Existing bytes under an identity conflict with the staged bytes.
    ConflictingChunk {
        /// Identity whose exact bytes conflict.
        identity: ChunkId,
    },
    /// Detector spans could not be admitted as one semantic layout.
    Layout(LayoutValidationError),
    /// Canonical layout identity calculation failed.
    LayoutEncoding(LayoutEncodeError),
}

impl fmt::Display for IngestionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => write!(formatter, "failed to read source bytes: {source}"),
            Self::InvalidReadCount { maximum, observed } => write!(
                formatter,
                "reader reported {observed} bytes for a {maximum}-byte buffer"
            ),
            Self::BlobHash(source) => source.fmt(formatter),
            Self::BlobIdentityMismatch { expected, observed } => write!(
                formatter,
                "staged bytes identify as {observed}, not required blob {expected}"
            ),
            Self::Chunking(source) => source.fmt(formatter),
            Self::ChunkHash(source) => source.fmt(formatter),
            Self::ChunkIdentityMismatch { expected, observed } => write!(
                formatter,
                "staged chunk identity mismatch: expected {expected:?}, observed {observed:?}"
            ),
            Self::BoundaryOutOfRange {
                feed_start,
                boundary,
                feed_length,
            } => write!(
                formatter,
                "chunk boundary {boundary} escaped {feed_length}-byte feed at {feed_start}"
            ),
            Self::MultipleBoundaries { feed_length } => write!(
                formatter,
                "more than one boundary appeared in a {feed_length}-byte feed"
            ),
            Self::StreamLengthOverflow { accepted, incoming } => write!(
                formatter,
                "stream length overflow after {accepted} bytes with {incoming} incoming bytes"
            ),
            Self::Allocation {
                target, requested, ..
            } => write!(
                formatter,
                "failed to reserve {requested} units for {target:?}"
            ),
            Self::CapacityExceeded {
                capacity,
                attempted,
            } => write!(
                formatter,
                "reference-store capacity {capacity} exceeded by {attempted} materialized bytes"
            ),
            Self::ConflictingChunk { identity } => {
                write!(formatter, "conflicting exact bytes for chunk {identity:?}")
            }
            Self::Layout(source) => source.fmt(formatter),
            Self::LayoutEncoding(source) => source.fmt(formatter),
        }
    }
}

impl Error for IngestionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::BlobHash(source) => Some(source),
            Self::Chunking(source) => Some(source),
            Self::ChunkHash(source) => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::Layout(source) => Some(source),
            Self::LayoutEncoding(source) => Some(source),
            Self::InvalidReadCount { .. }
            | Self::BlobIdentityMismatch { .. }
            | Self::ChunkIdentityMismatch { .. }
            | Self::BoundaryOutOfRange { .. }
            | Self::MultipleBoundaries { .. }
            | Self::StreamLengthOverflow { .. }
            | Self::CapacityExceeded { .. }
            | Self::ConflictingChunk { .. } => None,
        }
    }
}
