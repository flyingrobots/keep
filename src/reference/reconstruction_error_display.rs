//! Bounded diagnostic rendering for reconstruction failures.

use std::fmt::{self, Display};

use super::ReconstructionError;

impl fmt::Display for ReconstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayGroup::from(self).fmt(formatter)
    }
}

enum DisplayGroup<'a> {
    Presence(&'a ReconstructionError),
    Verification(&'a ReconstructionError),
    Output(&'a ReconstructionError),
}

impl DisplayGroup<'_> {
    fn fmt(self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Presence(error) => format_presence(formatter, error),
            Self::Verification(error) => format_verification(formatter, error),
            Self::Output(error) => format_output(formatter, error),
        }
    }
}

impl<'a> From<&'a ReconstructionError> for DisplayGroup<'a> {
    fn from(error: &'a ReconstructionError) -> Self {
        match error {
            ReconstructionError::BlobMissing { .. }
            | ReconstructionError::LayoutMissing { .. }
            | ReconstructionError::LayoutDecode(_)
            | ReconstructionError::LayoutEncoding(_) => Self::Presence(error),
            ReconstructionError::ChunkMissing { .. }
            | ReconstructionError::ChunkHash { .. }
            | ReconstructionError::ChunkIdentityMismatch { .. }
            | ReconstructionError::BlobHash(_)
            | ReconstructionError::BlobIdentityMismatch { .. } => Self::Verification(error),
            ReconstructionError::WriteZero { .. }
            | ReconstructionError::InvalidWriteCount { .. }
            | ReconstructionError::Write { .. }
            | ReconstructionError::WrittenLengthOverflow { .. }
            | ReconstructionError::WrittenLengthMismatch { .. } => Self::Output(error),
        }
    }
}

fn format_presence(formatter: &mut fmt::Formatter<'_>, error: &ReconstructionError) -> fmt::Result {
    match error {
        ReconstructionError::BlobMissing { requested } => {
            write!(formatter, "blob {requested} is absent")
        }
        ReconstructionError::LayoutMissing { requested } => {
            write!(formatter, "layout {requested} is absent")
        }
        ReconstructionError::LayoutDecode(source) => source.fmt(formatter),
        ReconstructionError::LayoutEncoding(source) => source.fmt(formatter),
        ReconstructionError::ChunkMissing { .. }
        | ReconstructionError::ChunkHash { .. }
        | ReconstructionError::ChunkIdentityMismatch { .. }
        | ReconstructionError::BlobHash(_)
        | ReconstructionError::BlobIdentityMismatch { .. }
        | ReconstructionError::WriteZero { .. }
        | ReconstructionError::InvalidWriteCount { .. }
        | ReconstructionError::Write { .. }
        | ReconstructionError::WrittenLengthOverflow { .. }
        | ReconstructionError::WrittenLengthMismatch { .. } => Err(fmt::Error),
    }
}

fn format_verification(
    formatter: &mut fmt::Formatter<'_>,
    error: &ReconstructionError,
) -> fmt::Result {
    match error {
        ReconstructionError::ChunkMissing {
            layout,
            index,
            requested,
        } => write!(
            formatter,
            "layout {layout} entry {index} is missing chunk {requested:?}"
        ),
        ReconstructionError::ChunkHash {
            layout,
            index,
            expected,
            source,
        } => write!(
            formatter,
            "layout {layout} entry {index} chunk {expected:?} cannot be verified: {source}"
        ),
        ReconstructionError::ChunkIdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => write!(
            formatter,
            "layout {layout} entry {index} expected chunk {expected:?}, observed {observed:?}"
        ),
        ReconstructionError::BlobHash(source) => source.fmt(formatter),
        ReconstructionError::BlobIdentityMismatch {
            layout,
            expected,
            observed,
        } => write!(
            formatter,
            "layout {layout} reconstructs {observed}, not named blob {expected}"
        ),
        ReconstructionError::BlobMissing { .. }
        | ReconstructionError::LayoutMissing { .. }
        | ReconstructionError::LayoutDecode(_)
        | ReconstructionError::LayoutEncoding(_)
        | ReconstructionError::WriteZero { .. }
        | ReconstructionError::InvalidWriteCount { .. }
        | ReconstructionError::Write { .. }
        | ReconstructionError::WrittenLengthOverflow { .. }
        | ReconstructionError::WrittenLengthMismatch { .. } => Err(fmt::Error),
    }
}

fn format_output(formatter: &mut fmt::Formatter<'_>, error: &ReconstructionError) -> fmt::Result {
    match error {
        ReconstructionError::WriteZero {
            layout,
            bytes_written,
        } => write!(
            formatter,
            "output stopped after {bytes_written} authenticated bytes for layout {layout}"
        ),
        ReconstructionError::InvalidWriteCount {
            layout,
            maximum,
            observed,
            bytes_written,
        } => write!(
            formatter,
            "writer reported {observed} bytes for a {maximum}-byte buffer after \
             {bytes_written} authenticated bytes of layout {layout}"
        ),
        ReconstructionError::Write {
            layout,
            bytes_written,
            source,
        } => write!(
            formatter,
            "failed after writing {bytes_written} authenticated bytes of layout {layout}: {source}"
        ),
        ReconstructionError::WrittenLengthOverflow {
            layout,
            bytes_written,
            incoming,
        } => write!(
            formatter,
            "written length overflow for layout {layout} after {bytes_written} bytes with \
             {incoming} incoming bytes"
        ),
        ReconstructionError::WrittenLengthMismatch {
            layout,
            expected,
            observed,
        } => write!(
            formatter,
            "layout {layout} wrote {observed} authenticated bytes, expected {expected}"
        ),
        ReconstructionError::BlobMissing { .. }
        | ReconstructionError::LayoutMissing { .. }
        | ReconstructionError::LayoutDecode(_)
        | ReconstructionError::LayoutEncoding(_)
        | ReconstructionError::ChunkMissing { .. }
        | ReconstructionError::ChunkHash { .. }
        | ReconstructionError::ChunkIdentityMismatch { .. }
        | ReconstructionError::BlobHash(_)
        | ReconstructionError::BlobIdentityMismatch { .. } => Err(fmt::Error),
    }
}
