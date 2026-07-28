//! Total diagnostic rendering for reconstruction failures.

use std::fmt::{self, Display};
use std::io;

use crate::{
    BlobId, BlobLength, ChunkHashError, ChunkId, ChunkingError, LayoutId, ProfileBoundary,
    StorageProfileId,
};

use super::ReconstructionError;

impl Display for ReconstructionError {
    // Keep the exhaustive arms directly auditable within the 60-line limit.
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlobMissing { requested } => write!(f, "blob {requested} is absent"),
            Self::LayoutMissing { requested } => write!(f, "layout {requested} is absent"),
            Self::LayoutDecode(source) => source.fmt(f),
            Self::LayoutEncoding(source) => source.fmt(f),
            Self::ChunkMissing { layout, index, requested } =>
                format_chunk_missing(f, *layout, *index, *requested),
            Self::ChunkHash { layout, index, expected, source } =>
                format_chunk_hash(f, *layout, *index, *expected, source),
            Self::ChunkIdentityMismatch { layout, index, expected, observed } =>
                format_chunk_mismatch(f, *layout, *index, *expected, *observed),
            Self::BlobHash(source) => source.fmt(f),
            Self::BlobIdentityMismatch { layout, expected, observed } =>
                format_blob_mismatch(f, *layout, *expected, *observed),
            Self::ProfileVerifierUnavailable { layout, profile } =>
                format_profile_unavailable(f, *layout, *profile),
            Self::ProfileChunking { layout, source } => format_profile_chunking(f, *layout, source),
            Self::ProfileBoundaryMismatch { layout, index, expected, observed } =>
                format_profile_mismatch(f, *layout, *index, *expected, *observed),
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
        "layout {layout} entry {index} is missing chunk {requested:?}"
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
        "layout {layout} entry {index} chunk {expected:?} cannot be verified: {source}"
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
        "layout {layout} entry {index} expected chunk {expected:?}, observed {observed:?}"
    )
}

fn format_blob_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    expected: BlobId,
    observed: BlobId,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} reconstructs {observed}, not named blob {expected}"
    )
}

fn format_profile_unavailable(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    profile: StorageProfileId,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} has no reconstruction verifier for registered profile {profile}"
    )
}

fn format_profile_chunking(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    source: &ChunkingError,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} storage-profile replay failed: {source}"
    )
}

fn format_profile_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    index: usize,
    expected: Option<ProfileBoundary>,
    observed: Option<ProfileBoundary>,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} profile boundary {index} expected {}, observed {}",
        BoundaryDisplay(expected),
        BoundaryDisplay(observed)
    )
}

struct BoundaryDisplay(Option<ProfileBoundary>);

impl Display for BoundaryDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(boundary) => boundary.fmt(formatter),
            None => formatter.write_str("no boundary"),
        }
    }
}

fn format_write_zero(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: BlobLength,
) -> fmt::Result {
    write!(
        formatter,
        "output stopped after {bytes_written} authenticated bytes for layout {layout}"
    )
}

fn format_invalid_write(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    maximum: usize,
    observed: usize,
    bytes_written: BlobLength,
) -> fmt::Result {
    write!(
        formatter,
        "writer reported {observed} bytes for a {maximum}-byte buffer after \
         {bytes_written} authenticated bytes of layout {layout}"
    )
}

fn format_write_error(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: BlobLength,
    source: &io::Error,
) -> fmt::Result {
    write!(
        formatter,
        "failed after writing {bytes_written} authenticated bytes of layout {layout}: {source}"
    )
}

fn format_length_overflow(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    bytes_written: u64,
    incoming: usize,
) -> fmt::Result {
    write!(
        formatter,
        "written length overflow for layout {layout} after {bytes_written} bytes with \
         {incoming} incoming bytes"
    )
}

fn format_length_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    layout: LayoutId,
    expected: BlobLength,
    observed: BlobLength,
) -> fmt::Result {
    write!(
        formatter,
        "layout {layout} wrote {observed} authenticated bytes, expected {expected}"
    )
}
