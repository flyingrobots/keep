//! This module owns semantic exact-range authentication and emission failures.

use super::{ChunkVerificationError, OutputWriteError};
use crate::{ByteLength, ByteRange, ChunkId, LayoutId, RangePlanError};

#[derive(Debug)]
pub(crate) enum RangeReadFailure {
    RangePlan(RangePlanError),
    Chunk(ChunkVerificationError),
    PlanEntriesUnavailable {
        first: usize,
        end: usize,
        available: usize,
    },
    ChunkSliceUnavailable {
        layout: LayoutId,
        index: usize,
        requested: ByteRange,
        chunk: ChunkId,
    },
    Output {
        layout: LayoutId,
        source: OutputWriteError,
    },
    WrittenLengthMismatch {
        layout: LayoutId,
        expected: ByteLength,
        observed: ByteLength,
    },
}
