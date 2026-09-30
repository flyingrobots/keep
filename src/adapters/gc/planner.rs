//! This boundary module owns deterministic GC planning.
//!
//! Planning is a pure comparison between one liveness snapshot and its
//! bounded inventory. It classifies every inventoried segment, refuses any
//! contradiction, and never guesses.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    GcLimits, GcLivenessSnapshot, GcPlan, GcPlanAmbiguity, GcPlanError, GcPlannedSegment,
    GcSegmentClassification, GcUnreachableEvidence,
};
use crate::adapters::SegmentDigest;

/// Plans one collection from `snapshot` within `limits`.
///
/// A segment the current catalog names is `Live` when a retained closure
/// reaches it and `NamedUnreachable` otherwise; neither is a candidate. An
/// unnamed segment is `Unreachable` only with superseding or disposition
/// evidence and `RecoveryProtected` without it. A superseded or disposed
/// segment absent from the inventory is reported as already retired.
///
/// # Errors
///
/// Returns [`GcPlanError::Ambiguous`] when the snapshot contradicts itself,
/// [`GcPlanError::CandidateLimit`] when more segments are collectible than
/// `limits` admit, or [`GcPlanError::RetainedRootOverflow`] when a segment's
/// retaining-root count exceeds `u32`.
pub fn plan_gc(snapshot: &GcLivenessSnapshot, limits: GcLimits) -> Result<GcPlan, GcPlanError> {
    require_consistent(snapshot)?;
    let reach = retained_reach(snapshot)?;
    let mut segments = BTreeMap::new();
    let mut candidate_count = 0_u32;
    for (segment, length) in snapshot.inventory() {
        let classification = classify(snapshot, &reach, *segment);
        if classification.is_candidate() {
            candidate_count =
                candidate_count
                    .checked_add(1)
                    .ok_or_else(|| GcPlanError::CandidateLimit {
                        limit: limits.candidates(),
                        observed: u32::MAX,
                    })?;
        }
        segments.insert(*segment, GcPlannedSegment::new(*length, classification));
    }
    if candidate_count > limits.candidates() {
        return Err(GcPlanError::CandidateLimit {
            limit: limits.candidates(),
            observed: candidate_count,
        });
    }
    let retired = snapshot
        .superseded()
        .union(snapshot.disposed())
        .filter(|segment| !snapshot.inventory().contains_key(segment))
        .copied()
        .collect::<BTreeSet<_>>();
    Ok(GcPlan::new(
        snapshot.coordinates(),
        segments,
        retired,
        candidate_count,
    ))
}

fn require_consistent(snapshot: &GcLivenessSnapshot) -> Result<(), GcPlanError> {
    if let Some(segment) = snapshot
        .named()
        .iter()
        .find(|segment| !snapshot.inventory().contains_key(segment))
    {
        return Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::NamedSegmentAbsent { segment: *segment },
        ));
    }
    for closure in snapshot.retained() {
        for segment in closure.segments() {
            if !snapshot.inventory().contains_key(segment) {
                return Err(GcPlanError::Ambiguous(
                    GcPlanAmbiguity::ClosureMemberAbsent {
                        namespace: closure.namespace(),
                        segment: *segment,
                    },
                ));
            }
            if !snapshot.named().contains(segment) {
                return Err(GcPlanError::Ambiguous(
                    GcPlanAmbiguity::ClosureMemberUnnamed {
                        namespace: closure.namespace(),
                        segment: *segment,
                    },
                ));
            }
        }
    }
    if let Some(segment) = snapshot.superseded().intersection(snapshot.named()).next() {
        return Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::SupersededSegmentNamed { segment: *segment },
        ));
    }
    if let Some(segment) = snapshot.disposed().intersection(snapshot.named()).next() {
        return Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::DisposedSegmentNamed { segment: *segment },
        ));
    }
    Ok(())
}

/// Counts, per segment, the retained roots whose closure reaches it.
fn retained_reach(
    snapshot: &GcLivenessSnapshot,
) -> Result<BTreeMap<SegmentDigest, u32>, GcPlanError> {
    let mut reach = BTreeMap::new();
    for closure in snapshot.retained() {
        for segment in closure.segments() {
            let count: &mut u32 = reach.entry(*segment).or_default();
            *count = count
                .checked_add(1)
                .ok_or(GcPlanError::RetainedRootOverflow { segment: *segment })?;
        }
    }
    Ok(reach)
}

fn classify(
    snapshot: &GcLivenessSnapshot,
    reach: &BTreeMap<SegmentDigest, u32>,
    segment: SegmentDigest,
) -> GcSegmentClassification {
    if snapshot.named().contains(&segment) {
        return match reach.get(&segment).copied() {
            Some(retained_roots) if retained_roots > 0 => {
                GcSegmentClassification::Live { retained_roots }
            }
            _ => GcSegmentClassification::NamedUnreachable,
        };
    }
    if snapshot.superseded().contains(&segment) {
        GcSegmentClassification::Unreachable(GcUnreachableEvidence::Superseded)
    } else if snapshot.disposed().contains(&segment) {
        GcSegmentClassification::Unreachable(GcUnreachableEvidence::Disposed)
    } else {
        GcSegmentClassification::RecoveryProtected
    }
}
