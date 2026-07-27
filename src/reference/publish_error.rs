//! Typed failures at the staged-to-visible transition.

use std::error::Error;
use std::fmt;

use crate::{ChunkId, LayoutId};

/// Failure while explicitly publishing staged reference-store work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishError {
    /// Concurrent intervening work exhausted the store capacity.
    CapacityExceeded {
        /// Configured materialized-byte capacity.
        capacity: usize,
        /// Materialized bytes required by the commit.
        attempted: usize,
    },
    /// Existing bytes under an identity conflict with staged bytes.
    ConflictingChunk {
        /// Conflicting chunk identity.
        identity: ChunkId,
    },
    /// Existing semantic layout under an identity conflicts with staged work.
    ConflictingLayout {
        /// Conflicting layout identity.
        identity: LayoutId,
    },
    /// A previously committed layout references an absent chunk.
    CommittedChunkMissing {
        /// Existing committed layout.
        layout: LayoutId,
        /// Exact chunk absent from committed state.
        chunk: ChunkId,
    },
}

impl fmt::Display for PublishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded {
                capacity,
                attempted,
            } => write!(
                formatter,
                "reference-store capacity {capacity} exceeded by {attempted} committed bytes"
            ),
            Self::ConflictingChunk { identity } => {
                write!(formatter, "conflicting exact bytes for chunk {identity:?}")
            }
            Self::ConflictingLayout { identity } => {
                write!(formatter, "conflicting semantic layout for {identity}")
            }
            Self::CommittedChunkMissing { layout, chunk } => {
                write!(
                    formatter,
                    "committed layout {layout} is missing chunk {chunk:?}"
                )
            }
        }
    }
}

impl Error for PublishError {}
