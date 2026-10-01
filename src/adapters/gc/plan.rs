//! This boundary module owns the immutable, inspectable GC plan.

use std::collections::{BTreeMap, BTreeSet};

use super::{GcLivenessCoordinates, GcSegmentClassification};
use crate::adapters::SegmentDigest;

/// One inventoried segment's length and classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcPlannedSegment {
    length: u64,
    classification: GcSegmentClassification,
}

impl GcPlannedSegment {
    pub(super) const fn new(length: u64, classification: GcSegmentClassification) -> Self {
        Self {
            length,
            classification,
        }
    }

    /// Returns the inventoried segment length.
    #[must_use]
    pub const fn length(self) -> u64 {
        self.length
    }

    /// Returns the classification.
    #[must_use]
    pub const fn classification(self) -> GcSegmentClassification {
        self.classification
    }
}

/// One collectible segment, in canonical digest order within its plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcPlannedCandidate {
    segment: SegmentDigest,
    length: u64,
}

impl GcPlannedCandidate {
    /// Returns the candidate segment digest.
    #[must_use]
    pub const fn segment(self) -> SegmentDigest {
        self.segment
    }

    /// Returns the candidate segment length.
    #[must_use]
    pub const fn length(self) -> u64 {
        self.length
    }
}

/// The deterministic classification of one physical inventory against one
/// liveness snapshot.
///
/// A plan is a statement, not an action. It names the coordinates it was
/// computed against, classifies every inventoried segment, and lists the
/// collectible candidates in canonical order. Nothing about it touches
/// storage; the retirement intent is derived from it under writer authority
/// and the exclusive reader lock, after execution has re-proven every
/// coordinate.
#[must_use = "a GC plan is evidence; discarding it collects nothing"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcPlan {
    coordinates: GcLivenessCoordinates,
    segments: BTreeMap<SegmentDigest, GcPlannedSegment>,
    retired: BTreeSet<SegmentDigest>,
    candidate_count: u32,
}

impl GcPlan {
    pub(super) const fn new(
        coordinates: GcLivenessCoordinates,
        segments: BTreeMap<SegmentDigest, GcPlannedSegment>,
        retired: BTreeSet<SegmentDigest>,
        candidate_count: u32,
    ) -> Self {
        Self {
            coordinates,
            segments,
            retired,
            candidate_count,
        }
    }

    /// Returns the coordinates the plan was computed against.
    #[must_use]
    pub const fn coordinates(&self) -> GcLivenessCoordinates {
        self.coordinates
    }

    /// Returns every inventoried segment with its classification.
    #[must_use]
    pub const fn segments(&self) -> &BTreeMap<SegmentDigest, GcPlannedSegment> {
        &self.segments
    }

    /// Returns one segment's classification, or `None` if it was not
    /// inventoried.
    #[must_use]
    pub fn classification(&self, segment: SegmentDigest) -> Option<GcSegmentClassification> {
        self.segments
            .get(&segment)
            .map(|planned| planned.classification())
    }

    /// Returns the segments that were superseded or disposed and are already
    /// absent from the inventory: retired by an earlier collection.
    #[must_use]
    pub const fn already_retired(&self) -> &BTreeSet<SegmentDigest> {
        &self.retired
    }

    /// Returns the collectible candidates in canonical digest order.
    pub fn candidates(&self) -> impl Iterator<Item = GcPlannedCandidate> + '_ {
        self.segments
            .iter()
            .filter(|(_, planned)| planned.classification().is_candidate())
            .map(|(segment, planned)| GcPlannedCandidate {
                segment: *segment,
                length: planned.length(),
            })
    }

    /// Returns the number of collectible candidates.
    #[must_use]
    pub const fn candidate_count(&self) -> u32 {
        self.candidate_count
    }

    /// Returns every segment at least one retained closure reaches.
    pub fn live_segments(&self) -> impl Iterator<Item = SegmentDigest> + '_ {
        self.segments
            .iter()
            .filter(|(_, planned)| {
                matches!(
                    planned.classification(),
                    GcSegmentClassification::Live { .. }
                )
            })
            .map(|(segment, _)| *segment)
    }
}
