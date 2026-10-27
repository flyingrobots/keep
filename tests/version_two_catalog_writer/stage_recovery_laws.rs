//! This module owns version-two segment and catalog stage recovery laws.

use std::fs;

use keep::{RecoveryEntryRole, RecoveryStage, RecoveryStageDiscardOutcome};

use super::durable_fixture;
use super::recovery_fixture::{complete, discard, stage_roles};
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, publish, republish, retain,
    second_blob,
};

#[test]
fn a_refused_candidate_leaves_a_segment_stage_recovery_completes() -> TestResult<()> {
    let first = first_blob();
    let second = second_blob();
    let sandbox = durable_fixture::build("v2-recover-refused-stage", &[&first])?;
    let root = sandbox.path();
    let mut publisher = open_publisher(root)?;
    let refused = publish(&mut publisher, root, &second, Predecessor::Dropped)?;
    drop(publisher);
    assert!(refused.is_err(), "a dropping candidate published");
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::SegmentStage]);

    let _receipt = complete(root, RecoveryStage::Segment)?;
    assert!(stage_roles(root)?.is_empty());
    let mut publisher = open_publisher(root)?;
    let _published = republish(&mut publisher, root)??;
    drop(publisher);
    retain(root, b"second", &second)?;
    assert_reads(root, &first)?;
    assert_reads(root, &second)
}

#[test]
fn an_incomplete_catalog_stage_is_discarded_by_version_two_recovery() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-recover-partial-catalog", &[&first])?;
    let root = sandbox.path();
    fs::write(root.join("staging/current.cat"), b"KEEP")?;
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::CatalogStage]);
    assert_reads(root, &first)?;

    let receipt = discard(root, RecoveryStage::Catalog)?;
    assert_eq!(receipt.outcome(), RecoveryStageDiscardOutcome::Removed);
    assert!(stage_roles(root)?.is_empty());
    let mut publisher = open_publisher(root)?;
    let _published = republish(&mut publisher, root)??;
    drop(publisher);
    assert_reads(root, &first)
}
