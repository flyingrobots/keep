//! Translation from shared read mechanics into public range failures.

use crate::{ByteLength, LayoutId};

use super::RangeReadError;
use super::chunk_verification::ChunkVerificationError;
use super::output_write::OutputWriteError;

pub(super) const fn range_chunk_error(error: ChunkVerificationError) -> RangeReadError {
    match error {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => RangeReadError::ChunkMissing {
            layout,
            index,
            requested,
        },
        ChunkVerificationError::Hash {
            layout,
            index,
            expected,
            source,
        } => RangeReadError::ChunkHash {
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
        } => RangeReadError::ChunkIdentityMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}

pub(super) fn range_output_error(layout: LayoutId, error: OutputWriteError) -> RangeReadError {
    match error {
        OutputWriteError::WriteZero { bytes_written } => RangeReadError::WriteZero {
            layout,
            bytes_written: ByteLength::new(bytes_written),
        },
        OutputWriteError::InvalidWriteCount {
            maximum,
            observed,
            bytes_written,
        } => RangeReadError::InvalidWriteCount {
            layout,
            maximum,
            observed,
            bytes_written: ByteLength::new(bytes_written),
        },
        OutputWriteError::Write {
            bytes_written,
            source,
        } => RangeReadError::Write {
            layout,
            bytes_written: ByteLength::new(bytes_written),
            source,
        },
        OutputWriteError::LengthOverflow {
            bytes_written,
            incoming,
        } => RangeReadError::WrittenLengthOverflow {
            layout,
            bytes_written: ByteLength::new(bytes_written),
            incoming,
        },
    }
}
