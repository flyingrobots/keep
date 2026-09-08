//! One-zero closure fixture construction and verification.
#![allow(
    clippy::redundant_pub_crate,
    reason = "private integration-test siblings share this closure fixture"
)]

use std::error::Error;

use keep::{
    BlobId, RetentionAnchor, RetentionClosureLimits, RetentionClosureVerificationError,
    RetentionNamespace, RetentionPolicy, RetentionRoot, RootGeneration, VerifiedRetentionClosure,
    verify_retention_closure,
};

use super::{
    BUNDLE_CATALOG_HEX, BUNDLE_HEAD_HEX, BUNDLE_SEGMENT_HEX, ONE_ZERO_BLOB, ONE_ZERO_LAYOUT,
    admitted_catalog, fixture, maximum_policy,
};

pub(super) fn root_with_limits(
    limits: RetentionClosureLimits,
    target: Option<BlobId>,
) -> Result<RetentionRoot, Box<dyn Error>> {
    let blob = target.map_or_else(|| ONE_ZERO_BLOB.parse(), Ok)?;
    Ok(RetentionRoot::new(
        RetentionNamespace::try_from(b"adversarial".as_slice())?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            keep::RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            limits,
        ),
        None,
        vec![RetentionAnchor::new(blob, ONE_ZERO_LAYOUT.parse()?)],
    )?)
}

pub(super) fn verify_bundle(
    root: &RetentionRoot,
) -> Result<Result<VerifiedRetentionClosure, RetentionClosureVerificationError>, Box<dyn Error>> {
    verify_fixture(
        root,
        BUNDLE_SEGMENT_HEX,
        BUNDLE_CATALOG_HEX,
        BUNDLE_HEAD_HEX,
    )
}

pub(super) fn verify_fixture(
    root: &RetentionRoot,
    segment_hex: &str,
    catalog_hex: &str,
    head_hex: &str,
) -> Result<Result<VerifiedRetentionClosure, RetentionClosureVerificationError>, Box<dyn Error>> {
    let segment_bytes = fixture(segment_hex)?;
    let catalog_bytes = fixture(catalog_hex)?;
    let head_bytes = fixture(head_hex)?;
    let segment = keep::AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog = admitted_catalog(&catalog_bytes, &segments)?;
    let head = keep::ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    Ok(verify_retention_closure(root, &snapshot))
}
