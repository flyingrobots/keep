//! Identified chunk span within one detector input stream.

use super::{ChunkId, ChunkLength, ChunkOffset};

/// One identified half-open chunk span emitted by [`FastCdc`](super::FastCdc).
///
/// The span describes bytes `[offset, end)` from one detector stream. It does
/// not store or borrow those bytes and is not a physical storage coordinate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkSpan {
    offset: ChunkOffset,
    end: ChunkOffset,
    id: ChunkId,
}

impl ChunkSpan {
    pub(super) const fn new(offset: ChunkOffset, end: ChunkOffset, id: ChunkId) -> Self {
        Self { offset, end, id }
    }

    /// Returns the absolute inclusive start coordinate.
    #[must_use]
    pub const fn offset(self) -> ChunkOffset {
        self.offset
    }

    /// Returns the absolute exclusive end coordinate.
    #[must_use]
    pub const fn end(self) -> ChunkOffset {
        self.end
    }

    /// Returns the exact span length.
    #[must_use]
    pub const fn length(self) -> ChunkLength {
        self.id.length()
    }

    /// Returns the identity calculated from the exact span bytes.
    #[must_use]
    pub const fn id(self) -> ChunkId {
        self.id
    }
}
