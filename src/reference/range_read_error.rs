//! Typed failures while reading one exact logical byte range.

use std::error::Error;
use std::io;

use crate::{
    BlobId, ByteLength, ByteRange, ChunkHashError, ChunkId, LayoutDecodeError, LayoutEncodeError,
    LayoutId, RangePlanError,
};

/// Failure while planning, authenticating, or emitting an exact byte range.
#[derive(Debug)]
pub enum RangeReadError {
    /// No committed layout names the requested blob.
    BlobMissing {
        /// Requested logical identity.
        requested: BlobId,
    },
    /// The requested committed layout is absent.
    LayoutMissing {
        /// Requested canonical layout identity.
        requested: LayoutId,
    },
    /// Supplied canonical layout bytes failed bounded decoding or admission.
    LayoutDecode(LayoutDecodeError),
    /// A supplied admitted layout could not produce its canonical identity.
    LayoutEncoding(LayoutEncodeError),
    /// The requested range could not be mapped onto the admitted layout.
    RangePlan(RangePlanError),
    /// A selected layout entry references an absent chunk.
    ChunkMissing {
        /// Layout being read.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Missing exact chunk identity.
        requested: ChunkId,
    },
    /// Selected stored bytes could not form a lawful chunk identity.
    ChunkHash {
        /// Layout being read.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Expected exact chunk identity.
        expected: ChunkId,
        /// Exact hashing failure.
        source: ChunkHashError,
    },
    /// Selected stored bytes do not match their named chunk identity.
    ChunkIdentityMismatch {
        /// Layout being read.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Identity named by the layout.
        expected: ChunkId,
        /// Identity calculated from stored bytes.
        observed: ChunkId,
    },
    /// A validated plan did not select an available entry interval.
    PlanEntriesUnavailable {
        /// First selected entry index.
        first: usize,
        /// Exclusive selected entry end.
        end: usize,
        /// Available layout entry count.
        available: usize,
    },
    /// A verified chunk could not supply the planned relative slice.
    ChunkSliceUnavailable {
        /// Layout being read.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Requested logical range.
        requested: ByteRange,
        /// Complete verified chunk identity.
        chunk: ChunkId,
    },
    /// The output refused to make progress.
    WriteZero {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated range bytes already written.
        bytes_written: ByteLength,
    },
    /// A broken writer reported more bytes than were supplied.
    InvalidWriteCount {
        /// Layout being emitted.
        layout: LayoutId,
        /// Supplied remaining byte count.
        maximum: usize,
        /// Count reported by the writer.
        observed: usize,
        /// Authenticated range bytes already written.
        bytes_written: ByteLength,
    },
    /// The output returned an I/O error other than interruption.
    Write {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated range bytes already written.
        bytes_written: ByteLength,
        /// Original output error.
        source: io::Error,
    },
    /// Checked output-byte accounting overflowed.
    WrittenLengthOverflow {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated range bytes already written.
        bytes_written: ByteLength,
        /// Newly accepted output count.
        incoming: usize,
    },
    /// The output count disagrees with the requested range length.
    WrittenLengthMismatch {
        /// Layout emitted.
        layout: LayoutId,
        /// Requested byte count.
        expected: ByteLength,
        /// Authenticated bytes reported written.
        observed: ByteLength,
    },
}

impl Error for RangeReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutDecode(source) => Some(source),
            Self::LayoutEncoding(source) => Some(source),
            Self::RangePlan(source) => Some(source),
            Self::ChunkHash { source, .. } => Some(source),
            Self::Write { source, .. } => Some(source),
            Self::BlobMissing { .. }
            | Self::LayoutMissing { .. }
            | Self::ChunkMissing { .. }
            | Self::ChunkIdentityMismatch { .. }
            | Self::PlanEntriesUnavailable { .. }
            | Self::ChunkSliceUnavailable { .. }
            | Self::WriteZero { .. }
            | Self::InvalidWriteCount { .. }
            | Self::WrittenLengthOverflow { .. }
            | Self::WrittenLengthMismatch { .. } => None,
        }
    }
}
