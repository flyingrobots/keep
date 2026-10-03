//! This module owns lossless outward mapping of chunk and output failures.

use super::ReconstructionError;
use crate::authenticated_read::{ChunkVerificationError, OutputWriteError};
use crate::{BlobLength, LayoutId};

pub(super) const fn reconstruction_chunk_error(
    error: ChunkVerificationError,
) -> ReconstructionError {
    match error {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => ReconstructionError::ChunkMissing {
            layout,
            index,
            requested,
        },
        ChunkVerificationError::Hash {
            layout,
            index,
            expected,
            source,
        } => ReconstructionError::ChunkHash {
            layout,
            index,
            expected,
            source,
        },
        ChunkVerificationError::IdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => ReconstructionError::ChunkIdentityMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}

pub(super) fn reconstruction_output_error(
    layout: LayoutId,
    error: OutputWriteError,
) -> ReconstructionError {
    match error {
        OutputWriteError::WriteZero { bytes_written } => ReconstructionError::WriteZero {
            layout,
            bytes_written: BlobLength::new(bytes_written),
        },
        OutputWriteError::InvalidWriteCount {
            maximum,
            observed,
            bytes_written,
        } => ReconstructionError::InvalidWriteCount {
            layout,
            maximum,
            observed,
            bytes_written: BlobLength::new(bytes_written),
        },
        OutputWriteError::Write {
            bytes_written,
            source,
        } => ReconstructionError::Write {
            layout,
            bytes_written: BlobLength::new(bytes_written),
            source,
        },
        OutputWriteError::LengthOverflow {
            bytes_written,
            incoming,
        } => ReconstructionError::WrittenLengthOverflow {
            layout,
            bytes_written: BlobLength::new(bytes_written),
            incoming,
        },
    }
}
