//! Logical identity admitted by a segment-record header.

use crate::{ChunkId, LayoutId};

/// Logical identity and record kind admitted from a segment-record header.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SegmentRecordIdentity {
    /// Exact nonempty chunk identity.
    Chunk(ChunkId),
    /// Exact canonical flat-layout identity.
    Layout(LayoutId),
}

impl SegmentRecordIdentity {
    pub(super) fn payload_length(self) -> u64 {
        match self {
            Self::Chunk(id) => u64::from(id.length().get()),
            Self::Layout(id) => id.plan_length().get(),
        }
    }
}
