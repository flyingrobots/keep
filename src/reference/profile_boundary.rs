//! Compact semantic coordinate for profile-replay diagnostics.

use crate::{ChunkLength, ChunkOffset, ChunkSpan, LayoutEntry};

/// One storage-profile boundary coordinate without retained content identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileBoundary {
    offset: ChunkOffset,
    length: ChunkLength,
}

impl ProfileBoundary {
    /// Returns the absolute inclusive start coordinate.
    #[must_use]
    pub const fn offset(self) -> ChunkOffset {
        self.offset
    }

    /// Returns the exact boundary length.
    #[must_use]
    pub const fn length(self) -> ChunkLength {
        self.length
    }
}

impl From<LayoutEntry> for ProfileBoundary {
    fn from(entry: LayoutEntry) -> Self {
        Self {
            offset: entry.offset(),
            length: entry.chunk_id().length(),
        }
    }
}

impl From<ChunkSpan> for ProfileBoundary {
    fn from(span: ChunkSpan) -> Self {
        Self {
            offset: span.offset(),
            length: span.length(),
        }
    }
}
