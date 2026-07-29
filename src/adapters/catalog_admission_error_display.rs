//! Human-readable catalog-admission diagnostics.

use std::error::Error;
use std::fmt;

use super::CatalogAdmissionError;

impl fmt::Display for CatalogAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog { source } => {
                write!(formatter, "catalog revalidation failed: {source}")
            }
            Self::EntryCountHostWidth { observed } => {
                write!(
                    formatter,
                    "catalog entry count {observed} exceeds host width"
                )
            }
            Self::SegmentCountOutOfBounds { maximum, observed } => write!(
                formatter,
                "catalog admits at most {maximum} segments, observed {observed}"
            ),
            Self::Allocation {
                phase, requested, ..
            } => write!(
                formatter,
                "catalog {phase} allocation failed for {requested} elements"
            ),
            Self::DuplicateSegment { .. } => {
                formatter.write_str("duplicate admitted segment digest")
            }
            Self::MissingSegment { .. } => formatter.write_str("catalog segment is missing"),
            Self::UnreferencedSegment { .. } => {
                formatter.write_str("admitted segment is not referenced by the catalog")
            }
            Self::Segment { source, .. } => {
                write!(formatter, "catalog segment revalidation failed: {source}")
            }
            Self::LocationNotTopLevel {
                record_offset,
                record_length,
                ..
            } => write!(
                formatter,
                "catalog location {record_offset}+{record_length} is not a top-level record"
            ),
            Self::RecordIdentityMismatch { .. } => {
                formatter.write_str("catalog and record identities disagree")
            }
            Self::RecordChecksumMismatch { .. } => {
                formatter.write_str("catalog and record checksums disagree")
            }
        }
    }
}

impl Error for CatalogAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Catalog { source } => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::Segment { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
