//! This boundary module owns GC planning refusals.

use std::fmt;

use crate::RetentionNamespaceDigest;
use crate::adapters::SegmentDigest;

/// A snapshot whose evidence contradicts itself: planning refuses rather than
/// guessing, and nothing is collected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcPlanAmbiguity {
    /// The current catalog names a segment the inventory lacks.
    NamedSegmentAbsent {
        /// The missing segment.
        segment: SegmentDigest,
    },
    /// A retained closure reaches a segment the current catalog does not name.
    ClosureMemberUnnamed {
        /// The retaining namespace.
        namespace: RetentionNamespaceDigest,
        /// The unnamed segment.
        segment: SegmentDigest,
    },
    /// A retained closure reaches a segment the inventory lacks.
    ClosureMemberAbsent {
        /// The retaining namespace.
        namespace: RetentionNamespaceDigest,
        /// The missing segment.
        segment: SegmentDigest,
    },
    /// A segment is both superseded and named by the current catalog.
    SupersededSegmentNamed {
        /// The contradictory segment.
        segment: SegmentDigest,
    },
    /// A segment has a retirement disposition yet the current catalog names it.
    DisposedSegmentNamed {
        /// The contradictory segment.
        segment: SegmentDigest,
    },
}

/// Failure to plan one collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcPlanError {
    /// The snapshot contradicts itself.
    Ambiguous(GcPlanAmbiguity),
    /// More segments are collectible than the limit admits.
    CandidateLimit {
        /// The admitted ceiling.
        limit: u32,
        /// The collectible count.
        observed: u32,
    },
    /// More retained roots reach one segment than the classification counts.
    RetainedRootOverflow {
        /// The over-retained segment.
        segment: SegmentDigest,
    },
}

impl fmt::Display for GcPlanAmbiguity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NamedSegmentAbsent { .. } => {
                "the current catalog names a segment absent from the inventory"
            }
            Self::ClosureMemberUnnamed { .. } => {
                "a retained closure reaches a segment the current catalog does not name"
            }
            Self::ClosureMemberAbsent { .. } => {
                "a retained closure reaches a segment absent from the inventory"
            }
            Self::SupersededSegmentNamed { .. } => {
                "a superseded segment is still named by the current catalog"
            }
            Self::DisposedSegmentNamed { .. } => {
                "a disposed segment is still named by the current catalog"
            }
        })
    }
}

impl fmt::Display for GcPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ambiguous(ambiguity) => write!(formatter, "GC plan refused: {ambiguity}"),
            Self::CandidateLimit { limit, observed } => write!(
                formatter,
                "GC plan has {observed} candidates, above the limit of {limit}"
            ),
            Self::RetainedRootOverflow { .. } => {
                formatter.write_str("retained-root count overflowed for one segment")
            }
        }
    }
}

impl std::error::Error for GcPlanAmbiguity {}

impl std::error::Error for GcPlanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ambiguous(ambiguity) => Some(ambiguity),
            Self::CandidateLimit { .. } | Self::RetainedRootOverflow { .. } => None,
        }
    }
}
