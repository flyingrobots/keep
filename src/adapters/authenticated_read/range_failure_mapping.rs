//! This module owns lossless translation of semantic range failures.

use super::RangeReadError;
use super::range_read_error_mapping::{range_chunk_error, range_output_error};
use crate::authenticated_read::RangeReadFailure;

impl From<RangeReadFailure> for RangeReadError {
    fn from(failure: RangeReadFailure) -> Self {
        match failure {
            RangeReadFailure::RangePlan(source) => Self::RangePlan(source),
            RangeReadFailure::Chunk(source) => range_chunk_error(source),
            RangeReadFailure::PlanEntriesUnavailable {
                first,
                end,
                available,
            } => Self::PlanEntriesUnavailable {
                first,
                end,
                available,
            },
            RangeReadFailure::ChunkSliceUnavailable {
                layout,
                index,
                requested,
                chunk,
            } => Self::ChunkSliceUnavailable {
                layout,
                index,
                requested,
                chunk,
            },
            RangeReadFailure::Output { layout, source } => range_output_error(layout, source),
            RangeReadFailure::WrittenLengthMismatch {
                layout,
                expected,
                observed,
            } => Self::WrittenLengthMismatch {
                layout,
                expected,
                observed,
            },
        }
    }
}
