//! Retention transition preflight laws.

mod support;

use std::error::Error;

use keep::{
    AdmittedCatalog, AdmittedRetentionRoot, AdmittedSegment, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, LayoutEntryLimit, RetentionClosureVerificationError,
    RetentionGenerationExpectation, RetentionTransitionError, RetentionTransitionPreflight,
    RetentionTransitionPreflightError, RootGeneration, SegmentReadPolicy, SegmentRecordIdentity,
    SegmentRecordLimit, preflight_retention_transition,
};
use support::{decode_hex, require_error};

const ROOT_HEX: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-head.hex");
const CHUNK_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CHUNK_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const CHUNK_HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");

#[test]
fn publish_preflight_binds_generation_and_closure_proofs() -> Result<(), Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let preflight = with_snapshot(
        BUNDLE_SEGMENT_HEX,
        BUNDLE_CATALOG_HEX,
        BUNDLE_HEAD_HEX,
        |snapshot| {
            preflight_retention_transition(
                RetentionGenerationExpectation::Absent,
                None,
                candidate,
                snapshot,
            )
        },
    )??;

    assert!(matches!(
        preflight,
        RetentionTransitionPreflight::Publish {
            candidate,
            closure,
        } if candidate.root().generation() == RootGeneration::INITIAL
            && closure.usage().node_count() == 2
    ));
    Ok(())
}

#[test]
fn stale_generation_refuses_before_missing_closure_evidence() -> Result<(), Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let result = with_snapshot(
        CHUNK_SEGMENT_HEX,
        CHUNK_CATALOG_HEX,
        CHUNK_HEAD_HEX,
        |snapshot| {
            preflight_retention_transition(
                RetentionGenerationExpectation::Current(RootGeneration::INITIAL),
                None,
                candidate,
                snapshot,
            )
        },
    )?;
    let error = require_error(result, "stale generation reached closure verification")?;

    assert!(matches!(
        error,
        RetentionTransitionPreflightError::Transition {
            source: RetentionTransitionError::StaleGeneration {
                expected: RetentionGenerationExpectation::Current(expected),
                observed: None,
            },
        } if expected == RootGeneration::INITIAL
    ));
    Ok(())
}

#[test]
fn valid_generation_preserves_missing_closure_member() -> Result<(), Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let result = with_snapshot(
        CHUNK_SEGMENT_HEX,
        CHUNK_CATALOG_HEX,
        CHUNK_HEAD_HEX,
        |snapshot| {
            preflight_retention_transition(
                RetentionGenerationExpectation::Absent,
                None,
                candidate,
                snapshot,
            )
        },
    )?;
    let error = require_error(result, "missing closure member passed preflight")?;
    let RetentionTransitionPreflightError::Closure { source } = error else {
        return Err("missing closure member reached the wrong preflight boundary".into());
    };

    assert!(matches!(
        *source,
        RetentionClosureVerificationError::MissingMember {
            identity: SegmentRecordIdentity::Layout(_),
        }
    ));
    Ok(())
}

#[test]
fn exact_retry_still_returns_current_closure_evidence() -> Result<(), Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let current = AdmittedRetentionRoot::decode(&root_bytes)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let preflight = with_snapshot(
        BUNDLE_SEGMENT_HEX,
        BUNDLE_CATALOG_HEX,
        BUNDLE_HEAD_HEX,
        |snapshot| {
            preflight_retention_transition(
                RetentionGenerationExpectation::Absent,
                Some(&current),
                candidate,
                snapshot,
            )
        },
    )??;

    assert!(matches!(
        preflight,
        RetentionTransitionPreflight::AlreadyCommitted { closure, .. }
            if closure.usage().physical_bytes() == 509
    ));
    Ok(())
}

fn with_snapshot<T>(
    segment_hex: &str,
    catalog_hex: &str,
    head_hex: &str,
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> Result<T, RetentionTransitionPreflightError>,
) -> Result<Result<T, RetentionTransitionPreflightError>, Box<dyn Error>> {
    let segment_bytes = fixture(segment_hex)?;
    let catalog_bytes = fixture(catalog_hex)?;
    let head_bytes = fixture(head_hex)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog = admitted_catalog(&catalog_bytes, &segments)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    Ok(operation(&snapshot))
}

fn admitted_catalog<'catalog, 'records>(
    catalog_bytes: &'catalog [u8],
    segments: &'records [AdmittedSegment<'records>],
) -> Result<AdmittedCatalog<'catalog, 'records>, Box<dyn Error>> {
    ChecksummedCatalog::decode(catalog_bytes)?
        .admit(segments)
        .map_err(Into::into)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
