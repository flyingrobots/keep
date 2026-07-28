//! Typed failures at the staged-to-visible transition.

use std::error::Error;
use std::fmt;

use crate::{BlobId, ChunkId, LayoutId};

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
    /// Staged work and the destination both lack a required chunk.
    StagedChunkMissing {
        /// Layout that cannot be published completely.
        layout: LayoutId,
        /// Exact chunk absent from both staged and destination state.
        chunk: ChunkId,
    },
    /// A previously committed layout references an absent chunk.
    CommittedChunkMissing {
        /// Existing committed layout.
        layout: LayoutId,
        /// Exact chunk absent from committed state.
        chunk: ChunkId,
    },
    /// A committed layout is absent from its target blob's visible index.
    CommittedLayoutIndexMissing {
        /// Existing layout missing from its target's index.
        layout: LayoutId,
    },
    /// A visible blob index references an absent committed layout.
    CommittedLayoutMissing {
        /// Indexed layout absent from committed layout state.
        layout: LayoutId,
    },
    /// A committed layout identity appears under another target blob.
    CommittedLayoutMisindexed {
        /// Layout indexed under the wrong target.
        layout: LayoutId,
        /// Incorrect target blob index.
        observed: BlobId,
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
            Self::StagedChunkMissing { layout, chunk } => write!(
                formatter,
                "staged layout {layout} and its destination are missing chunk {chunk:?}"
            ),
            Self::CommittedChunkMissing { layout, chunk } => {
                write!(
                    formatter,
                    "committed layout {layout} is missing chunk {chunk:?}"
                )
            }
            Self::CommittedLayoutIndexMissing { layout } => {
                write!(
                    formatter,
                    "committed layout {layout} is absent from its blob index"
                )
            }
            Self::CommittedLayoutMissing { layout } => {
                write!(
                    formatter,
                    "blob index references absent committed layout {layout}"
                )
            }
            Self::CommittedLayoutMisindexed { layout, observed } => write!(
                formatter,
                "committed layout {layout} is indexed under wrong blob {observed}"
            ),
        }
    }
}

impl Error for PublishError {}
