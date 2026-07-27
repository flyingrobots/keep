//! One semantic entry in an admitted flat layout.

use crate::{ChunkId, ChunkOffset, ChunkSpan};

/// One exact identified chunk at an absolute logical blob offset.
///
/// This value contains no physical location, storage handle, retention fact,
/// or borrowed chunk bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutEntry {
    offset: ChunkOffset,
    id: ChunkId,
}

impl LayoutEntry {
    pub(crate) const fn from_validated_parts(offset: ChunkOffset, id: ChunkId) -> Self {
        Self { offset, id }
    }

    /// Returns the absolute logical byte offset.
    #[must_use]
    pub const fn offset(self) -> ChunkOffset {
        self.offset
    }

    /// Returns the exact physical chunk identity.
    #[must_use]
    pub const fn chunk_id(self) -> ChunkId {
        self.id
    }
}

impl From<ChunkSpan> for LayoutEntry {
    fn from(span: ChunkSpan) -> Self {
        Self {
            offset: span.offset(),
            id: span.id(),
        }
    }
}
