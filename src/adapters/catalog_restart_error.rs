//! Published catalog restart-loading failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;
use std::io;

use super::{
    CatalogAdmissionError, CatalogDecodeError, CatalogRestartArtifact, CatalogRestartPhase,
    CatalogSnapshotError, PublicationHeadDecodeError, SegmentDigest, SegmentReadError,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Failure to reconstruct one exact published catalog snapshot.
#[derive(Debug)]
pub enum CatalogRestartError {
    /// One capability-relative filesystem operation failed.
    Io {
        /// Exact operation that failed.
        phase: CatalogRestartPhase,
        /// Preserved filesystem source.
        source: io::Error,
    },
    /// An opened protocol artifact was not a regular file.
    NotRegular {
        /// Artifact whose type was wrong.
        artifact: CatalogRestartArtifact,
    },
    /// An artifact length violated an exact or bounded expectation.
    Length {
        /// Artifact whose length was refused.
        artifact: CatalogRestartArtifact,
        /// Smallest accepted length.
        minimum: u64,
        /// Largest accepted length.
        maximum: u64,
        /// Exact observed length.
        observed: u64,
    },
    /// Exact length arithmetic could not be represented.
    LengthArithmetic {
        /// Artifact whose observation could not be represented.
        artifact: CatalogRestartArtifact,
        /// Length established before the overflow.
        expected: u64,
    },
    /// Host memory could not represent or reserve a bounded artifact.
    Allocation {
        /// Artifact being materialized.
        artifact: CatalogRestartArtifact,
        /// Exact requested byte count.
        byte_count: u64,
        /// Preserved allocation source when reservation was attempted.
        source: Option<TryReserveError>,
    },
    /// A host segment-index length could not be represented as a protocol count.
    SegmentIndexLength,
    /// Host memory could not reserve the exact segment index.
    SegmentIndexAllocation {
        /// Exact number of segment entries requested.
        segment_count: u64,
        /// Preserved allocation source.
        source: TryReserveError,
    },
    /// Aggregate retained segment bytes exceeded caller policy.
    RetainedSegmentBytes {
        /// Caller-selected maximum.
        maximum: u64,
        /// Exact attempted aggregate.
        observed: u64,
    },
    /// Aggregate retained-segment byte arithmetic overflowed.
    RetainedSegmentByteArithmetic {
        /// Bytes retained before the failing addition.
        current: u64,
        /// Bytes selected by the next segment.
        addition: u64,
    },
    /// Publication-head bytes were malformed or corrupt.
    Head {
        /// Preserved head decoder refusal.
        source: PublicationHeadDecodeError,
    },
    /// Catalog bytes were malformed, noncanonical, or corrupt.
    Catalog {
        /// Preserved catalog decoder refusal.
        source: CatalogDecodeError,
    },
    /// The selected catalog disagreed with the head coordinate.
    CatalogCoordinate {
        /// Generation required by the head.
        expected_generation: CatalogGeneration,
        /// Generation verified from the catalog.
        observed_generation: CatalogGeneration,
        /// Length required by the head.
        expected_length: CatalogLength,
        /// Length verified from the catalog.
        observed_length: CatalogLength,
        /// Digest required by the head.
        expected_digest: CatalogDigest,
        /// Digest verified from the catalog.
        observed_digest: CatalogDigest,
    },
    /// One selected segment was malformed or corrupt.
    Segment {
        /// Digest required by the catalog.
        expected: SegmentDigest,
        /// Preserved segment refusal.
        source: Box<SegmentReadError>,
    },
    /// A valid segment's content digest disagreed with its selected name.
    SegmentCoordinate {
        /// Digest required by the catalog.
        expected: SegmentDigest,
        /// Digest verified from bytes.
        observed: SegmentDigest,
    },
    /// Catalog locations failed exact segment-record admission.
    CatalogAdmission {
        /// Preserved admission refusal.
        source: Box<CatalogAdmissionError>,
    },
    /// Head and admitted catalog failed final snapshot binding.
    Snapshot {
        /// Preserved snapshot refusal.
        source: CatalogSnapshotError,
    },
}

impl CatalogRestartError {
    pub(super) const fn io(phase: CatalogRestartPhase, source: io::Error) -> Self {
        Self::Io { phase, source }
    }
}

impl fmt::Display for CatalogRestartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { phase, .. } => write!(formatter, "catalog restart {phase} failed"),
            Self::NotRegular { .. } => formatter.write_str("restart artifact is not regular"),
            Self::Length { .. } => formatter.write_str("restart artifact length is invalid"),
            Self::LengthArithmetic { .. } => {
                formatter.write_str("restart artifact length overflowed")
            }
            Self::Allocation { .. } => formatter.write_str("restart allocation failed"),
            Self::SegmentIndexLength => {
                formatter.write_str("restart segment index length is not representable")
            }
            Self::SegmentIndexAllocation { .. } => {
                formatter.write_str("restart segment index allocation failed")
            }
            Self::RetainedSegmentBytes { .. } => {
                formatter.write_str("retained segment bytes exceed restart policy")
            }
            Self::RetainedSegmentByteArithmetic { .. } => {
                formatter.write_str("retained segment byte arithmetic overflowed")
            }
            Self::Head { .. } => formatter.write_str("publication head admission failed"),
            Self::Catalog { .. } => formatter.write_str("catalog admission failed"),
            Self::CatalogCoordinate { .. } => {
                formatter.write_str("catalog disagrees with publication head")
            }
            Self::Segment { .. } => formatter.write_str("segment admission failed"),
            Self::SegmentCoordinate { .. } => {
                formatter.write_str("segment disagrees with catalog coordinate")
            }
            Self::CatalogAdmission { .. } => formatter.write_str("catalog record binding failed"),
            Self::Snapshot { .. } => formatter.write_str("restart snapshot binding failed"),
        }
    }
}

impl Error for CatalogRestartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Allocation {
                source: Some(source),
                ..
            }
            | Self::SegmentIndexAllocation { source, .. } => Some(source),
            Self::Head { source } => Some(source),
            Self::Catalog { source } => Some(source),
            Self::Segment { source, .. } => Some(source),
            Self::CatalogAdmission { source } => Some(source),
            Self::Snapshot { source } => Some(source),
            Self::NotRegular { .. }
            | Self::Length { .. }
            | Self::LengthArithmetic { .. }
            | Self::Allocation { source: None, .. }
            | Self::SegmentIndexLength
            | Self::RetainedSegmentBytes { .. }
            | Self::RetainedSegmentByteArithmetic { .. }
            | Self::CatalogCoordinate { .. }
            | Self::SegmentCoordinate { .. } => None,
        }
    }
}
