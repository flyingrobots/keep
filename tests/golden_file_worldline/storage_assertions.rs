//! Public storage-path replay and bounded reference-model laws.

use std::io::Cursor;

use keep::{
    BlobId, IngestionError, LayoutEntryLimit, ReconstructionError, ReferenceStore,
    ReferenceStoreCapacity,
};

use super::TestResult;
use super::harness_failure::HarnessFailure;
use super::identity_assertions::assert_named_bytes;
use super::identity_corpus::find_case;
use super::mutation_assertions::apply_mutation;
use super::reference_model::{MODEL_CAPACITY_BYTES, ReferenceModel, ReferenceModelError};
use super::scenario_corpus::{mutation_cases, scenario_steps};

#[test]
fn public_reference_store_executes_the_golden_worldline() -> TestResult {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(MODEL_CAPACITY_BYTES));
    for step in scenario_steps()? {
        let identity = find_case(step.identity_case)?.expected_id()?;
        match step.operation {
            "identify" => assert_identify_step(step.input_case, identity, step.expected_outcome)?,
            "admit-exact" => {
                assert_eq!(step.expected_outcome, "keep.content.admitted");
                let bytes = find_case(step.input_case)?.bytes()?;
                let mut source = Cursor::new(bytes);
                let published = store
                    .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
                    .commit(&mut store)?;
                assert_eq!(published.target(), identity);
            }
            "read-exact" => {
                assert_eq!(step.expected_outcome, "keep.content.exact");
                let expected = find_case(step.input_case)?.bytes()?;
                let mut observed = Vec::new();
                let receipt = store.reconstruct(identity, &mut observed)?;
                assert_eq!(receipt.target(), identity);
                assert_named_bytes(identity, &observed)?;
                assert_eq!(observed, expected);
            }
            "verify-claimed-content" => {
                assert_claimed_mismatch(&store, step.input_case, identity, step.expected_outcome)?;
            }
            "read-absent" => {
                assert_eq!(step.expected_outcome, "keep.content.absent");
                let mut observed = Vec::new();
                assert!(matches!(
                    store.reconstruct(identity, &mut observed),
                    Err(ReconstructionError::BlobMissing { requested })
                        if requested == identity
                ));
                assert!(observed.is_empty());
            }
            _ => return Err(HarnessFailure::corpus("unknown worldline operation")),
        }
    }
    Ok(())
}

fn assert_identify_step(input_case: &str, identity: BlobId, expected_outcome: &str) -> TestResult {
    assert_eq!(expected_outcome, "keep.identity.identified");
    assert_eq!(
        BlobId::hash_bytes(&find_case(input_case)?.bytes()?)?,
        identity
    );
    Ok(())
}

fn assert_claimed_mismatch(
    store: &ReferenceStore,
    input_case: &str,
    identity: BlobId,
    expected_outcome: &str,
) -> TestResult {
    assert_eq!(expected_outcome, "keep.content.mismatch");
    let bytes = find_case(input_case)?.bytes()?;
    let mut source = Cursor::new(bytes);
    let result = store.stage_expected(&mut source, identity, LayoutEntryLimit::MAXIMUM);
    assert!(matches!(
        result,
        Err(IngestionError::BlobIdentityMismatch { expected, .. })
            if expected == identity
    ));
    Ok(())
}

#[test]
fn reference_model_capacity_is_bounded_and_refusal_is_atomic() -> TestResult {
    let mut model = ReferenceModel::new();
    let capacity_filler = vec![0_u8; MODEL_CAPACITY_BYTES];
    let retained = model.admit_exact_materialized(&capacity_filler)?;
    assert_eq!(model.admit_exact_materialized(&capacity_filler)?, retained);

    let refused_bytes = [1_u8];
    let refused = BlobId::hash_bytes(&refused_bytes)?;
    let attempted = MODEL_CAPACITY_BYTES
        .checked_add(refused_bytes.len())
        .ok_or_else(|| HarnessFailure::corpus("model capacity test overflowed"))?;
    assert_eq!(
        model.admit_exact_materialized(&refused_bytes),
        Err(ReferenceModelError::CapacityExceeded {
            capacity: MODEL_CAPACITY_BYTES,
            attempted,
        })
    );
    assert_eq!(
        model.read_exact_materialized(retained)?,
        capacity_filler.as_slice()
    );
    assert_eq!(
        model.read_exact_materialized(refused),
        Err(ReferenceModelError::Absent { requested: refused })
    );
    Ok(())
}

#[test]
fn public_expected_ingestion_refuses_every_content_mutation() -> TestResult {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(MODEL_CAPACITY_BYTES));
    for mutation in mutation_cases()?
        .into_iter()
        .filter(|case| case.target_kind == "content")
    {
        let fixture = find_case(mutation.target_case)?;
        let identity = fixture.expected_id()?;
        let mut bytes = fixture.bytes()?;
        apply_mutation(&mut bytes, &mutation)?;
        assert_eq!(mutation.expected_outcome, "keep.content.mismatch");
        let mut source = Cursor::new(bytes);
        let result = store.stage_expected(&mut source, identity, LayoutEntryLimit::MAXIMUM);
        assert!(matches!(
            result,
            Err(IngestionError::BlobIdentityMismatch { expected, .. })
                if expected == identity
        ));
    }
    Ok(())
}
