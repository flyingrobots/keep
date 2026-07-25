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

/// Failure while advancing the deterministic streaming chunker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkingError {
    /// Accepting another byte would exceed the stream coordinate range.
    StreamLengthOverflow {
        /// Number of bytes accepted before the refused byte.
        accepted: ChunkOffset,
    },
    /// Candidate length accounting exceeded the registered profile maximum.
    ChunkLengthOverflow {
        /// Maximum candidate length admitted by the registered profile.
        maximum: ChunkLength,
    },
    /// The compiled Gear table is missing the entry for an input byte.
    MissingGearEntry {
        /// Input byte whose table entry was absent.
        byte: u8,
    },
    /// A boundary transition attempted to emit an empty chunk.
    EmptyBoundary {
        /// Coordinate where the invalid transition was detected.
        offset: ChunkOffset,
    },
}

impl fmt::Display for ChunkingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StreamLengthOverflow { accepted } => {
                write!(formatter, "stream length overflow after {accepted} bytes")
            }
            Self::ChunkLengthOverflow { maximum } => write!(
                formatter,
                "candidate length exceeded the profile maximum of {maximum} bytes"
            ),
            Self::MissingGearEntry { byte } => {
                write!(
                    formatter,
                    "compiled Gear table has no entry for byte {byte}"
                )
            }
            Self::EmptyBoundary { offset } => {
                write!(
                    formatter,
                    "boundary at offset {offset} would emit an empty chunk"
                )
            }
        }
    }
}

impl Error for ChunkingError {}
