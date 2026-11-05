//! Retention evidence that changes before `HEAD` replacement refuses the commit.

use std::error::Error;

use keep::{
    CatalogPublicationPhase, RetainedCatalogPublicationError as Refusal, RetentionEvidencePhase,
    RetentionHeadDecodeError,
};

use crate::catalog_fixture::{Candidate, with_candidate};
use crate::model_storage::{EvidenceRead, ModelStorage};
use crate::retention_fixture::{RetentionRecords, RootShape};
use crate::support::require_error;
use crate::{publish, stages_before_replacement};

/// Publishes the preserving candidate and requires a refusal before replacement.
fn refuse_at_commit(mut storage: ModelStorage) -> Result<Refusal, Box<dyn Error>> {
    let result = with_candidate(Candidate::Preserving, |call| publish(&mut storage, &call))?;
    let error = require_error(result, "changed retention evidence was committed")?;
    let replacement = stages_before_replacement()?;
    assert_eq!(storage.stages().len(), replacement);
    assert_eq!(
        storage.stages().last(),
        Some(&CatalogPublicationPhase::VerifyHeadView)
    );
    assert!(
        !storage
            .stages()
            .contains(&CatalogPublicationPhase::ReplaceHead)
    );
    assert_eq!(
        storage.reads().last(),
        Some(&(EvidenceRead::Head, replacement))
    );
    Ok(error)
}

#[test]
fn head_replaced_between_admission_and_commit_refuses() -> Result<(), Box<dyn Error>> {
    let records = RetentionRecords::golden()?;
    let expected = records.current_head()?;
    let successor = RetentionRecords::built(&[(b"later", RootShape::Released)])?;
    let observed = successor.current_head()?;
    let storage = ModelStorage::new(records).then_head(successor.head);

    let error = refuse_at_commit(storage)?;

    assert!(matches!(
        error,
        Refusal::RetentionChanged { expected: e, observed: ref o }
            if e == Some(expected) && **o == Some(observed)
    ));
    Ok(())
}

#[test]
fn head_removed_between_admission_and_commit_refuses() -> Result<(), Box<dyn Error>> {
    let records = RetentionRecords::golden()?;
    let expected = records.current_head()?;
    let storage = ModelStorage::new(records).then_head(None);

    let error = refuse_at_commit(storage)?;

    assert!(matches!(
        error,
        Refusal::RetentionChanged { expected: e, ref observed } if e == Some(expected) && observed.is_none()
    ));
    Ok(())
}

#[test]
fn head_published_between_admission_and_commit_refuses() -> Result<(), Box<dyn Error>> {
    let appeared = RetentionRecords::golden()?;
    let observed = appeared.current_head()?;
    let storage = ModelStorage::new(RetentionRecords::unretained()).then_head(appeared.head);

    let error = refuse_at_commit(storage)?;

    assert!(matches!(
        error,
        Refusal::RetentionChanged { expected: None, observed: ref o } if **o == Some(observed)
    ));
    Ok(())
}

#[test]
fn corrupt_head_at_commit_refuses_with_the_revalidation_phase() -> Result<(), Box<dyn Error>> {
    let records = RetentionRecords::golden()?;
    let mut corrupt = records.head.clone().ok_or("golden head is absent")?;
    let last = corrupt.last_mut().ok_or("golden head is empty")?;
    *last ^= 1;
    let storage = ModelStorage::new(records).then_head(Some(corrupt));

    let error = refuse_at_commit(storage)?;

    assert!(matches!(
        error,
        Refusal::HeadDecode {
            phase: RetentionEvidencePhase::RevalidateHead,
            source: RetentionHeadDecodeError::ChecksumMismatch { .. }
        }
    ));
    Ok(())
}

#[test]
fn failed_head_read_at_commit_refuses_with_the_revalidation_phase() -> Result<(), Box<dyn Error>> {
    let storage = ModelStorage::new(RetentionRecords::golden()?).failing_head_read(1);

    let error = refuse_at_commit(storage)?;

    assert!(matches!(
        error,
        Refusal::RetentionStorage {
            phase: RetentionEvidencePhase::RevalidateHead,
            ..
        }
    ));
    Ok(())
}
