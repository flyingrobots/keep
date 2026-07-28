//! Typed complete segment-record content admission failures.

use super::{LayoutDecodeError, SegmentRecordHeaderError};
use crate::{ChunkHashError, ChunkId};

/// A complete segment record failed logical content admission or preparation.
#[derive(Debug)]
pub enum SegmentRecordAdmissionError {
    /// Chunk identity calculation failed.
    ChunkHash {
        /// Precise chunk hashing failure.
        source: ChunkHashError,
    },
    /// A canonical record header could not be constructed.
    Header {
        /// Precise header construction failure.
        source: SegmentRecordHeaderError,
    },
    /// A decoded chunk payload does not match its declared identity.
    ChunkIdentityMismatch {
        /// Identity declared by the record header.
        expected: ChunkId,
        /// Identity calculated from the exact payload.
        observed: ChunkId,
    },
    /// Layout payload decoding or expected-identity verification failed.
    Layout {
        /// Precise bounded layout failure.
        source: LayoutDecodeError,
    },
    /// A payload host length cannot be represented by the wire coordinate.
    PayloadLengthHostWidth {
        /// Supplied payload host length.
        observed: usize,
    },
    /// Prepared payload length disagrees with its constructed header.
    PayloadLengthMismatch {
        /// Header payload length.
        expected: u64,
        /// Supplied payload length.
        observed: u64,
    },
    /// Checked complete-record framing arithmetic failed.
    RecordLengthArithmetic {
        /// Constructed complete record length.
        observed: u64,
    },
}
