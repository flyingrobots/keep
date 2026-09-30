//! GC planning laws: one law per classification, one per ambiguity, the
//! candidate limit, and the golden plan for the frozen version-2 store.

#[path = "plan_model_tests.rs"]
mod plan_model_tests;

use std::collections::BTreeSet;
use std::error::Error;

use super::{
    GcLimits, GcLimitsError, GcLivenessCoordinates, GcLivenessSnapshot, GcLivenessSnapshotError,
    GcPlanAmbiguity, GcPlanError, GcRetainedClosure, GcRetentionState, GcSegmentClassification,
    GcUnreachableEvidence, plan_gc,
};
use crate::adapters::retention::AdmittedRetentionRoot;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedSegment, ChecksummedCatalog, ChecksummedPublicationHead, SegmentDigest,
    SegmentReadPolicy, SegmentRecordLimit,
};
use crate::{
    CatalogDigest, CatalogGeneration, LayoutEntryLimit, LivenessGeneration, RetentionClosureDigest,
    RetentionManifestDigest, RetentionNamespaceDigest, RetentionRootDigest, RootGeneration,
    verify_retention_closure,
};

const GOLDEN_PLAN: &str = include_str!("../../../conformance/segment-store/v2/gc-plan.tsv");
const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-head.hex");
const ROOT_HEX: &str = include_str!("../../../conformance/segment-store/v2/one-anchor-root.hex");
const MANIFEST_DIGEST_HEX: &str =
    "f46b96a2bf3379320cf59e8af15b9d108de06025c415307b28953714bd7a80eb";

pub(super) fn segment(seed: u8) -> SegmentDigest {
    SegmentDigest::from_validated([seed; 32])
}

pub(super) fn namespace(seed: u8) -> RetentionNamespaceDigest {
    RetentionNamespaceDigest::from_hash([seed; 32])
}

pub(super) fn coordinates() -> Result<GcLivenessCoordinates, Box<dyn Error>> {
    Ok(GcLivenessCoordinates::new(
        CatalogGeneration::new(3)?,
        CatalogDigest::from_validated([0x33; 32]),
        GcRetentionState::Published {
            generation: LivenessGeneration::new(2)?,
            manifest_digest: RetentionManifestDigest::from_hash([0x44; 32]),
        },
    ))
}

pub(super) fn closure(
    seed: u8,
    segments: &[SegmentDigest],
) -> Result<GcRetainedClosure, Box<dyn Error>> {
    Ok(GcRetainedClosure::new(
        namespace(seed),
        RootGeneration::new(1)?,
        RetentionRootDigest::from_hash([seed; 32]),
        RetentionClosureDigest::from_verified([seed; 32]),
        segments.iter().copied().collect(),
    ))
}

fn snapshot(named: &[u8], unnamed: &[u8]) -> Result<GcLivenessSnapshot, Box<dyn Error>> {
    let mut snapshot = GcLivenessSnapshot::new(coordinates()?);
    for seed in named {
        snapshot.inventory_segment(segment(*seed), u64::from(*seed))?;
        snapshot.name_segment(segment(*seed));
    }
    for seed in unnamed {
        snapshot.inventory_segment(segment(*seed), u64::from(*seed))?;
    }
    Ok(snapshot)
}

#[test]
fn named_segments_reached_by_retained_closures_are_live_with_their_root_count()
-> Result<(), Box<dyn Error>> {
    let mut snapshot = snapshot(&[1, 2], &[])?;
    snapshot.retain(closure(10, &[segment(1)])?)?;
    snapshot.retain(closure(11, &[segment(1), segment(2)])?)?;

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(
        plan.classification(segment(1)),
        Some(GcSegmentClassification::Live { retained_roots: 2 })
    );
    assert_eq!(
        plan.classification(segment(2)),
        Some(GcSegmentClassification::Live { retained_roots: 1 })
    );
    assert_eq!(plan.candidate_count(), 0);
    assert_eq!(
        plan.live_segments().collect::<Vec<_>>(),
        [segment(1), segment(2)]
    );
    Ok(())
}

#[test]
fn a_named_segment_no_root_reaches_is_named_unreachable_and_never_a_candidate()
-> Result<(), Box<dyn Error>> {
    let snapshot = snapshot(&[1], &[])?;

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(
        plan.classification(segment(1)),
        Some(GcSegmentClassification::NamedUnreachable)
    );
    assert_eq!(plan.candidates().count(), 0);
    Ok(())
}

#[test]
fn an_unnamed_segment_without_release_evidence_is_recovery_protected() -> Result<(), Box<dyn Error>>
{
    let snapshot = snapshot(&[], &[5])?;

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(
        plan.classification(segment(5)),
        Some(GcSegmentClassification::RecoveryProtected)
    );
    assert_eq!(plan.candidate_count(), 0);
    Ok(())
}

#[test]
fn superseded_and_disposed_unnamed_segments_are_the_only_candidates() -> Result<(), Box<dyn Error>>
{
    let mut snapshot = snapshot(&[1], &[5, 6, 7])?;
    snapshot.supersede_segment(segment(5));
    snapshot.dispose_segment(segment(6));
    snapshot.supersede_segment(segment(7));
    snapshot.dispose_segment(segment(7));

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(
        plan.classification(segment(5)),
        Some(GcSegmentClassification::Unreachable(
            GcUnreachableEvidence::Superseded
        ))
    );
    assert_eq!(
        plan.classification(segment(6)),
        Some(GcSegmentClassification::Unreachable(
            GcUnreachableEvidence::Disposed
        ))
    );
    assert_eq!(
        plan.classification(segment(7)),
        Some(GcSegmentClassification::Unreachable(
            GcUnreachableEvidence::Superseded
        )),
        "superseding evidence is reported before a disposition"
    );
    let candidates: Vec<_> = plan
        .candidates()
        .map(|c| (c.segment(), c.length()))
        .collect();
    assert_eq!(
        candidates,
        [(segment(5), 5), (segment(6), 6), (segment(7), 7)]
    );
    assert_eq!(plan.candidate_count(), 3);
    Ok(())
}

#[test]
fn superseded_or_disposed_segments_absent_from_the_inventory_are_already_retired()
-> Result<(), Box<dyn Error>> {
    let mut snapshot = snapshot(&[1], &[])?;
    snapshot.supersede_segment(segment(8));
    snapshot.dispose_segment(segment(9));

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(
        plan.already_retired(),
        &[segment(8), segment(9)]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(plan.classification(segment(8)), None);
    assert_eq!(plan.candidate_count(), 0);
    Ok(())
}

#[test]
fn every_contradiction_refuses_the_plan() -> Result<(), Box<dyn Error>> {
    let mut named_absent = GcLivenessSnapshot::new(coordinates()?);
    named_absent.name_segment(segment(1));
    assert_eq!(
        plan_gc(&named_absent, GcLimits::MAXIMUM),
        Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::NamedSegmentAbsent {
                segment: segment(1)
            }
        ))
    );

    let mut member_unnamed = snapshot(&[1], &[2])?;
    member_unnamed.retain(closure(10, &[segment(2)])?)?;
    assert_eq!(
        plan_gc(&member_unnamed, GcLimits::MAXIMUM),
        Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::ClosureMemberUnnamed {
                namespace: namespace(10),
                segment: segment(2)
            }
        ))
    );

    let mut member_absent = snapshot(&[1], &[])?;
    member_absent.retain(closure(10, &[segment(3)])?)?;
    assert_eq!(
        plan_gc(&member_absent, GcLimits::MAXIMUM),
        Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::ClosureMemberAbsent {
                namespace: namespace(10),
                segment: segment(3)
            }
        ))
    );

    let mut superseded_named = snapshot(&[1], &[])?;
    superseded_named.supersede_segment(segment(1));
    assert_eq!(
        plan_gc(&superseded_named, GcLimits::MAXIMUM),
        Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::SupersededSegmentNamed {
                segment: segment(1)
            }
        ))
    );

    let mut disposed_named = snapshot(&[1], &[])?;
    disposed_named.dispose_segment(segment(1));
    assert_eq!(
        plan_gc(&disposed_named, GcLimits::MAXIMUM),
        Err(GcPlanError::Ambiguous(
            GcPlanAmbiguity::DisposedSegmentNamed {
                segment: segment(1)
            }
        ))
    );
    Ok(())
}

#[test]
fn the_candidate_limit_refuses_rather_than_truncates() -> Result<(), Box<dyn Error>> {
    let mut snapshot = snapshot(&[], &[5, 6])?;
    snapshot.supersede_segment(segment(5));
    snapshot.supersede_segment(segment(6));

    assert_eq!(
        plan_gc(&snapshot, GcLimits::new(1)?),
        Err(GcPlanError::CandidateLimit {
            limit: 1,
            observed: 2
        })
    );
    assert_eq!(GcLimits::new(0), Err(GcLimitsError::Zero));
    assert_eq!(
        GcLimits::new(65_537),
        Err(GcLimitsError::AboveMaximum { requested: 65_537 })
    );
    assert_eq!(GcLimits::MAXIMUM.candidates(), 65_536);
    Ok(())
}

#[test]
fn an_empty_inventory_plans_nothing() -> Result<(), Box<dyn Error>> {
    let snapshot = GcLivenessSnapshot::new(coordinates()?);

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert!(plan.segments().is_empty());
    assert_eq!(plan.candidate_count(), 0);
    assert_eq!(plan.coordinates(), coordinates()?);
    Ok(())
}

#[test]
fn a_snapshot_refuses_duplicate_inventory_and_namespaces() -> Result<(), Box<dyn Error>> {
    let mut snapshot = snapshot(&[1], &[])?;
    assert_eq!(
        snapshot.inventory_segment(segment(1), 1),
        Err(GcLivenessSnapshotError::DuplicateInventory {
            segment: segment(1)
        })
    );
    snapshot.retain(closure(10, &[segment(1)])?)?;
    assert_eq!(
        snapshot.retain(closure(10, &[])?),
        Err(GcLivenessSnapshotError::DuplicateNamespace {
            namespace: namespace(10)
        })
    );
    Ok(())
}

/// The frozen version-two store: the one-zero bundle catalog at generation
/// one, retained by the frozen one-anchor root under the frozen generation-one
/// manifest. Its plan is the golden ledger row for row.
#[test]
fn the_golden_version_two_store_plans_one_live_segment() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let catalog_bytes = fixture(BUNDLE_CATALOG_HEX)?;
    let head_bytes = fixture(BUNDLE_HEAD_HEX)?;
    let admitted_segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segment_digest = admitted_segment.digest();
    let segment_length = admitted_segment.segment_length();
    let segments = [admitted_segment];
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?.admit(&segments)?;
    let catalog_snapshot = ChecksummedPublicationHead::decode(&head_bytes)?.admit(catalog)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let verified = verify_retention_closure(root.root(), &catalog_snapshot)?;
    let manifest_digest = RetentionManifestDigest::from_hash(array(MANIFEST_DIGEST_HEX)?);
    let mut snapshot = GcLivenessSnapshot::new(GcLivenessCoordinates::new(
        catalog_snapshot.generation(),
        catalog_snapshot.catalog_digest(),
        GcRetentionState::Published {
            generation: LivenessGeneration::new(1)?,
            manifest_digest,
        },
    ));
    snapshot.inventory_segment(segment_digest, segment_length)?;
    snapshot.name_segment(segment_digest);
    snapshot.retain(GcRetainedClosure::new(
        root.root().namespace().digest(),
        root.root().generation(),
        root.digest(),
        verified.digest(),
        BTreeSet::from([segment_digest]),
    ))?;

    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    let rows: Vec<_> = GOLDEN_PLAN.lines().skip(2).collect();
    assert_eq!(rows.len(), plan.segments().len());
    for (row, (digest, planned)) in rows.iter().zip(plan.segments()) {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(
            fields.first().copied(),
            Some(hex(digest.as_bytes()).as_str())
        );
        assert_eq!(
            fields.get(1).copied(),
            Some(planned.length().to_string().as_str())
        );
        assert_eq!(
            fields.get(2).copied(),
            Some(planned.classification().identifier())
        );
        let GcSegmentClassification::Live { retained_roots } = planned.classification() else {
            return Err("the golden store's only segment must be live".into());
        };
        assert_eq!(
            fields.get(3).copied(),
            Some(retained_roots.to_string().as_str())
        );
    }
    assert_eq!(plan.candidate_count(), 0);
    Ok(())
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

fn array(hex: &str) -> Result<[u8; 32], Box<dyn Error>> {
    <[u8; 32]>::try_from(decode_hex(hex)?).map_err(|_| "digest must be 32 bytes".into())
}

fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
