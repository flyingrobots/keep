//! This module owns segment and catalog stage recovery laws.

use std::fs;

use keep::{
    FilesystemRecoveryAdmission, FilesystemRecoveryInventoryReader,
    FilesystemRecoverySegmentResumer, RecoveryEntryRole, RecoveryStage,
    RecoveryStageDiscardOutcome, SegmentRecordLimit, StagedSegment, admit_recovery_stage_bytes,
    assess_recovery_stage, execute_recovery_segment_resume, plan_recovery_segment_resume,
};

use super::durable_fixture;
use super::recovery_fixture::{complete, discard, stage_roles};
use super::successor_catalog::blob_records;
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, publish, republish, retain,
    second_blob, segment_policy,
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
fn an_incomplete_catalog_stage_is_discarded_by_stage_recovery() -> TestResult<()> {
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

#[test]
fn a_reusable_segment_prefix_resumes_under_recovery_admission() -> TestResult<()> {
    let first = first_blob();
    let second = second_blob();
    let sandbox = durable_fixture::build("v2-recover-resume", &[&first])?;
    let root = sandbox.path();
    let identified = durable_fixture::identify(&second)?;
    let mut records = blob_records(&second, &identified)?.into_iter();
    let publisher = open_publisher(root)?;
    let prefix = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?
    .append(records.next().ok_or("blob has no record")?)?;
    drop(prefix);
    drop(publisher);
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::SegmentStage]);

    let request = resume_plan(root)?;
    let admission = FilesystemRecoveryAdmission::reopen(root)?;
    let resumer = FilesystemRecoverySegmentResumer::open(admission)?;
    let mut continued = execute_recovery_segment_resume(resumer, request)?;
    for record in records {
        continued = continued.append(record)?;
    }
    drop(continued.seal()?);
    let _receipt = complete(root, RecoveryStage::Segment)?;
    let mut publisher = open_publisher(root)?;
    let _published = republish(&mut publisher, root)??;
    drop(publisher);
    retain(root, b"second", &second)?;
    assert_reads(root, &first)?;
    assert_reads(root, &second)
}

fn resume_plan(root: &std::path::Path) -> TestResult<keep::RecoverySegmentResumeRequest> {
    let reader = FilesystemRecoveryInventoryReader::open(root)?;
    let evidence = reader.fingerprint_stage(RecoveryStage::Segment)?;
    let bytes = fs::read(root.join("staging/current.seg"))?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Segment, evidence, &bytes)?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())?;
    Ok(plan_recovery_segment_resume(&assessment, segment_policy())?)
}
