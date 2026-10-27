//! This module owns writer refusals that must precede every mutation.

use std::fs;
use std::io;
use std::path::Path;

use keep::{RetainedCatalogPublicationError, RetentionCurrentStateRefusal, RetentionEvidencePhase};

use super::durable_fixture;
use super::writer_fixture::{
    Predecessor, StoreWitness, TestResult, assert_reads, first_blob, open_publisher, publish,
    second_blob, witness,
};

/// Requires that only the caller's sealed segment stage was added.
fn assert_only_caller_stage_added(before: &StoreWitness, root: &Path) -> TestResult<()> {
    let mut after = witness(root)?;
    let position = after
        .iter()
        .position(|(path, _)| path == "staging/current.seg")
        .ok_or("the caller's sealed segment stage is absent")?;
    after.remove(position);
    assert_eq!(before, &after, "refusal mutated the store");
    Ok(())
}

fn current_state_refusal(source: &io::Error) -> Option<&RetentionCurrentStateRefusal> {
    source
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<RetentionCurrentStateRefusal>())
}

#[test]
fn a_candidate_that_drops_retained_content_refuses_before_any_mutation() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-writer-dropped", &[&first])?;
    let root = sandbox.path();
    let before = witness(root)?;
    let mut publisher = open_publisher(root)?;
    let outcome = publish(&mut publisher, root, &second_blob(), Predecessor::Dropped)?;
    drop(publisher);

    let Err(RetainedCatalogPublicationError::Closure { entry, .. }) = outcome else {
        return Err(format!("dropping retained content was not refused: {outcome:?}").into());
    };
    assert_eq!(entry.root_generation().get(), 1);
    assert_only_caller_stage_added(&before, root)?;
    assert_reads(root, &first)
}

#[test]
fn absent_retention_head_contradicted_by_pool_entries_refuses() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-writer-headless", &[&first])?;
    let root = sandbox.path();
    fs::remove_file(root.join("retention/HEAD"))?;
    let before = witness(root)?;
    let mut publisher = open_publisher(root)?;
    let outcome = publish(&mut publisher, root, &second_blob(), Predecessor::Dropped)?;
    drop(publisher);

    let Err(RetainedCatalogPublicationError::RetentionStorage { phase, source }) = outcome else {
        return Err(format!("contradicted head absence was admitted: {outcome:?}").into());
    };
    assert_eq!(phase, RetentionEvidencePhase::AdmitHead);
    assert!(matches!(
        current_state_refusal(&source),
        Some(RetentionCurrentStateRefusal::HeadAbsentWithArtifacts)
    ));
    assert_only_caller_stage_added(&before, root)
}

#[test]
fn a_fixed_retention_stage_requires_retention_recovery_first() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-writer-retention-stage", &[&first])?;
    let root = sandbox.path();
    fs::write(
        root.join("retention/root.next"),
        b"interrupted retention stage",
    )?;
    let before = witness(root)?;
    let mut publisher = open_publisher(root)?;
    let outcome = publish(&mut publisher, root, &second_blob(), Predecessor::Kept)?;
    drop(publisher);

    let Err(RetainedCatalogPublicationError::RetentionStorage { phase, source }) = outcome else {
        return Err(format!("a retained retention stage was ignored: {outcome:?}").into());
    };
    assert_eq!(phase, RetentionEvidencePhase::AdmitHead);
    assert!(matches!(
        current_state_refusal(&source),
        Some(RetentionCurrentStateRefusal::RetainedStage)
    ));
    assert_only_caller_stage_added(&before, root)
}
