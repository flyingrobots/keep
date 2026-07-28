//! Typed failures while reconstructing exact named bytes.

use std::error::Error;
use std::io;

use crate::{
    BlobHashError, BlobId, BlobLength, ChunkHashError, ChunkId, ChunkingError, LayoutDecodeError,
    LayoutEncodeError, LayoutId, ProfileBoundary, StorageProfileId,
};

/// Failure while verifying or emitting a committed logical blob.
#[derive(Debug)]
pub enum ReconstructionError {
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
    /// A committed layout references an absent chunk.
    ChunkMissing {
        /// Layout being reconstructed.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Missing exact chunk identity.
        requested: ChunkId,
    },
    /// Stored chunk bytes could not form a lawful chunk identity.
    ChunkHash {
        /// Layout being reconstructed.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Expected exact chunk identity.
        expected: ChunkId,
        /// Exact hashing failure.
        source: ChunkHashError,
    },
    /// Stored chunk bytes do not match the identity named by the layout.
    ChunkIdentityMismatch {
        /// Layout being reconstructed.
        layout: LayoutId,
        /// Zero-based semantic entry index.
        index: usize,
        /// Identity named by the layout.
        expected: ChunkId,
        /// Identity calculated from stored bytes.
        observed: ChunkId,
    },
    /// Logical identity calculation failed.
    BlobHash(BlobHashError),
    /// Fully reconstructed bytes do not match the layout target.
    BlobIdentityMismatch {
        /// Layout whose complete byte stream was verified.
        layout: LayoutId,
        /// Logical identity named by the layout.
        expected: BlobId,
        /// Logical identity calculated from all verified chunks.
        observed: BlobId,
    },
    /// No verifier is implemented for the admitted storage profile.
    ProfileVerifierUnavailable {
        /// Layout whose registered profile could not be verified.
        layout: LayoutId,
        /// Registered profile without a reconstruction verifier.
        profile: StorageProfileId,
    },
    /// Replaying the registered storage profile failed.
    ProfileChunking {
        /// Layout whose registered profile was replayed.
        layout: LayoutId,
        /// Exact detector failure.
        source: ChunkingError,
    },
    /// Replayed profile boundaries differ from the committed layout entries.
    ProfileBoundaryMismatch {
        /// Layout whose registered profile was replayed.
        layout: LayoutId,
        /// Zero-based boundary index.
        index: usize,
        /// Entry committed by the layout, or absence for an extra boundary.
        expected: Option<ProfileBoundary>,
        /// Entry emitted by profile replay, or absence for a missing boundary.
        observed: Option<ProfileBoundary>,
    },
    /// The output refused to make progress.
    WriteZero {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated bytes already written.
        bytes_written: BlobLength,
    },
    /// A broken writer reported more bytes than were supplied.
    InvalidWriteCount {
        /// Layout being emitted.
        layout: LayoutId,
        /// Supplied remaining byte count.
        maximum: usize,
        /// Count reported by the writer.
        observed: usize,
        /// Authenticated bytes already written before this call.
        bytes_written: BlobLength,
    },
    /// The output returned an I/O error other than interruption.
    Write {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated bytes already written.
        bytes_written: BlobLength,
        /// Original output error.
        source: io::Error,
    },
    /// Checked output-byte accounting overflowed.
    WrittenLengthOverflow {
        /// Layout being emitted.
        layout: LayoutId,
        /// Authenticated bytes already written.
        bytes_written: BlobLength,
        /// Newly accepted output count.
        incoming: usize,
    },
    /// The output count disagrees with the already verified target length.
    WrittenLengthMismatch {
        /// Layout emitted.
        layout: LayoutId,
        /// Target logical length.
        expected: BlobLength,
        /// Authenticated bytes reported written.
        observed: BlobLength,
    },
}

impl Error for ReconstructionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChunkHash { source, .. } => Some(source),
            Self::BlobHash(source) => Some(source),
            Self::ProfileChunking { source, .. } => Some(source),
            Self::LayoutDecode(source) => Some(source),
            Self::LayoutEncoding(source) => Some(source),
            Self::Write { source, .. } => Some(source),
            Self::BlobMissing { .. }
            | Self::LayoutMissing { .. }
            | Self::ChunkMissing { .. }
            | Self::ChunkIdentityMismatch { .. }
            | Self::BlobIdentityMismatch { .. }
            | Self::ProfileVerifierUnavailable { .. }
            | Self::ProfileBoundaryMismatch { .. }
            | Self::WriteZero { .. }
            | Self::InvalidWriteCount { .. }
            | Self::WrittenLengthOverflow { .. }
            | Self::WrittenLengthMismatch { .. } => None,
        }
    }
}
