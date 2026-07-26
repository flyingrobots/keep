//! Chunk identity and boundary-detection failures.

use std::error::Error;
use std::fmt;

use super::{ChunkLength, ChunkOffset};

/// Failure while calculating a canonical chunk identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkHashError {
    /// A chunk identity cannot name the empty byte sequence.
    Empty,
    /// The input slice length cannot be represented by the version-1 field.
    InputLengthOutOfRange {
        /// Platform slice length that could not be represented.
        observed: usize,
    },
}

impl fmt::Display for ChunkHashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a chunk identity cannot name empty content"),
            Self::InputLengthOutOfRange { observed } => write!(
                formatter,
                "input length {observed} cannot be represented by ChunkId v1"
            ),
        }
    }
}

impl Error for ChunkHashError {}

/// Terminal failure while advancing the deterministic streaming chunker.
///
/// Each variant reports how many bytes the failed `FastCdc::feed` call
/// accepted before refusal. A failed detector repeats the original error from
/// later `feed` and `finish` calls and must be discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkingError {
    /// Accepting another byte would exceed the stream coordinate range.
    StreamLengthOverflow {
        /// Number of bytes accepted before the refused byte.
        accepted: ChunkOffset,
        /// Bytes accepted from the call that first failed.
        call_bytes_accepted: usize,
    },
    /// Candidate length accounting exceeded the registered profile maximum.
    ChunkLengthOverflow {
        /// Maximum candidate length admitted by the registered profile.
        maximum: ChunkLength,
        /// Candidate length that the refused byte would have produced.
        attempted: u64,
        /// Bytes accepted from the call that first failed.
        call_bytes_accepted: usize,
    },
    /// The compiled Gear table is missing the entry for an input byte.
    MissingGearEntry {
        /// Input byte whose table entry was absent.
        byte: u8,
        /// Bytes accepted from the call that first failed.
        call_bytes_accepted: usize,
    },
    /// A boundary transition attempted to emit an empty chunk.
    EmptyBoundary {
        /// Coordinate where the invalid transition was detected.
        offset: ChunkOffset,
        /// Bytes accepted from the call that first failed.
        call_bytes_accepted: usize,
    },
    /// Internal feed-range accounting escaped the caller-owned slice.
    FeedRangeInvariant {
        /// Inclusive range start requested from the feed slice.
        start: usize,
        /// Exclusive range end requested from the feed slice.
        end: usize,
        /// Actual feed-slice length.
        input_length: usize,
        /// Bytes accepted from the call that first failed.
        call_bytes_accepted: usize,
    },
}

impl fmt::Display for ChunkingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StreamLengthOverflow {
                accepted,
                call_bytes_accepted,
            } => write!(
                formatter,
                "stream length overflow after {accepted} bytes; the failed call accepted \
                 {call_bytes_accepted} bytes"
            ),
            Self::ChunkLengthOverflow {
                maximum,
                attempted,
                call_bytes_accepted,
            } => write!(
                formatter,
                "candidate length {attempted} exceeded the profile maximum of {maximum} bytes; \
                 the failed call accepted {call_bytes_accepted} bytes"
            ),
            Self::MissingGearEntry {
                byte,
                call_bytes_accepted,
            } => write!(
                formatter,
                "compiled Gear table has no entry for byte {byte}; the failed call accepted \
                 {call_bytes_accepted} bytes"
            ),
            Self::EmptyBoundary {
                offset,
                call_bytes_accepted,
            } => write!(
                formatter,
                "boundary at offset {offset} would emit an empty chunk; the failed call accepted \
                 {call_bytes_accepted} bytes"
            ),
            Self::FeedRangeInvariant {
                start,
                end,
                input_length,
                call_bytes_accepted,
            } => write!(
                formatter,
                "feed hash range {start}..{end} escaped {input_length} input bytes; the failed \
                 call accepted {call_bytes_accepted} bytes"
            ),
        }
    }
}

impl Error for ChunkingError {}
