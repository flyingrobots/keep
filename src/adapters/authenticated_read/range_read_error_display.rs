//! Total diagnostic rendering for exact range-read failures.

use std::fmt::{self, Display};
use std::io;

use crate::{ByteLength, ByteRange, ChunkHashError, ChunkId, LayoutId};

use super::RangeReadError;

impl Display for RangeReadError {
    // Keep the exhaustive arms directly auditable within the 60-line limit.
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlobMissing { requested } => write!(f, "blob {requested} is absent"),
            Self::LayoutMissing { requested } => write!(f, "layout {requested} is absent"),
            Self::LayoutDecode(source) => source.fmt(f),
            Self::LayoutEncoding(source) => source.fmt(f),
            Self::RangePlan(source) => source.fmt(f),
            Self::ChunkMissing { layout, index, requested } =>
                format_chunk_missing(f, *layout, *index, *requested),
            Self::ChunkHash { layout, index, expected, source } =>
                format_chunk_hash(f, *layout, *index, *expected, source),
            Self::ChunkIdentityMismatch { layout, index, expected, observed } =>
                format_chunk_mismatch(f, *layout, *index, *expected, *observed),
            Self::PlanEntriesUnavailable { first, end, available } =>
                format_plan_entries(f, *first, *end, *available),
            Self::ChunkSliceUnavailable { layout, index, requested, chunk } =>
                format_chunk_slice(f, *layout, *index, *requested, *chunk),
            Self::WriteZero { layout, bytes_written } =>
                format_write_zero(f, *layout, *bytes_written),
            Self::InvalidWriteCount { layout, maximum, observed, bytes_written } =>
                format_invalid_write(f, *layout, *maximum, *observed, *bytes_written),
            Self::Write { layout, bytes_written, source } =>
                format_write_error(f, *layout, *bytes_written, source),
            Self::WrittenLengthOverflow { layout, bytes_written, incoming } =>
                format_length_overflow(f, *layout, *bytes_written, *incoming),
            Self::WrittenLengthMismatch { layout, expected, observed } =>
                format_length_mismatch(f, *layout, *expected, *observed),
        }
    }
}

fn format_chunk_missing(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    index: usize,
    requested: ChunkId,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} range entry {index} is missing chunk {requested:?}"
    )
}

fn format_chunk_hash(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    index: usize,
    expected: ChunkId,
    source: &ChunkHashError,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} range entry {index} chunk {expected:?} cannot be verified: {source}"
    )
}

fn format_chunk_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    index: usize,
    expected: ChunkId,
    observed: ChunkId,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} range entry {index} expected chunk {expected:?}, observed {observed:?}"
    )
}

fn format_plan_entries(
    formatter: &mut fmt::Formatter<'_>,
    first: usize,
    end: usize,
    available: usize,
) -> fmt::Result {
    write!(
        formatter,
        "range plan entries [{first}, {end}) exceed {available} available entries"
    )
}

fn format_chunk_slice(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    index: usize,
    requested: ByteRange,
    chunk: ChunkId,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} entry {index} chunk {chunk:?} cannot supply range [{}, {})",
        requested.offset(),
        requested.end()
    )
}

fn format_write_zero(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: ByteLength,
) -> fmt::Result {
    write!(
        formatter,
        "range output stopped after {bytes_written} authenticated bytes for layout {layout}"
    )
}

fn format_invalid_write(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    maximum: usize,
    observed: usize,
    bytes_written: ByteLength,
) -> fmt::Result {
    write!(
        formatter,
        "range writer reported {observed} bytes for a {maximum}-byte buffer after \
         {bytes_written} authenticated bytes of layout {layout}"
    )
}

fn format_write_error(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: ByteLength,
    source: &io::Error,
) -> fmt::Result {
    write!(
        formatter,
        "range write failed after {bytes_written} authenticated bytes of layout {layout}: {source}"
    )
}

fn format_length_overflow(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: ByteLength,
    incoming: usize,
) -> fmt::Result {
    write!(
        formatter,
        "range written length overflow for layout {layout} after {bytes_written} bytes with \
         {incoming} incoming bytes"
    )
}

fn format_length_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    expected: ByteLength,
    observed: ByteLength,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} range wrote {observed} authenticated bytes, expected {expected}"
    )
}
