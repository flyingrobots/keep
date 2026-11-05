//! Retained-closure catalog publication laws.

#[path = "retained_catalog_publication/catalog_fixture.rs"]
pub mod catalog_fixture;
#[path = "retained_catalog_publication/closure_model_laws.rs"]
mod closure_model_laws;
#[path = "retained_catalog_publication/closure_refusal_laws.rs"]
mod closure_refusal_laws;
#[path = "retained_catalog_publication/evidence_refusal_laws.rs"]
mod evidence_refusal_laws;
#[path = "retention_closure/memory_stage.rs"]
mod memory_stage;
#[path = "retained_catalog_publication/model_storage.rs"]
pub mod model_storage;
#[path = "catalog_publication/recording_storage.rs"]
pub mod recording_storage;
#[path = "retained_catalog_publication/retention_fixture.rs"]
pub mod retention_fixture;
#[path = "retained_catalog_publication/revalidation_laws.rs"]
mod revalidation_laws;
mod support;

use std::error::Error;

use catalog_fixture::{Candidate, CandidateCall, with_candidate};
use keep::{
    CatalogGeneration, CatalogPublicationError, CatalogPublicationOutcome, CatalogPublicationPhase,
    RetainedCatalogPublicationError, RetainedCatalogPublicationReceipt, SegmentPublication,
    publish_retained_catalog_generation,
};
use model_storage::{EvidenceRead, ModelStorage};
use recording_storage::{EXPECTED_WITH_SEGMENT, RecordingStorage};
use retention_fixture::RetentionRecords;
use support::require_error;

/// Catalog-only protocol order: every phase except the five segment phases.
fn catalog_only_phases() -> Vec<CatalogPublicationPhase> {
    EXPECTED_WITH_SEGMENT
        .iter()
        .copied()
        .filter(|phase| !is_segment_phase(*phase))
        .collect()
}

const fn is_segment_phase(phase: CatalogPublicationPhase) -> bool {
    matches!(
        phase,
        CatalogPublicationPhase::LinkSegment
            | CatalogPublicationPhase::VerifySegmentPool
            | CatalogPublicationPhase::SynchronizeSegments
            | CatalogPublicationPhase::RemoveSegmentStage
            | CatalogPublicationPhase::SynchronizeStagingAfterSegment
    )
}

/// Number of catalog-only stages that precede `HEAD` replacement.
fn stages_before_replacement() -> Result<usize, Box<dyn Error>> {
    catalog_only_phases()
        .iter()
        .position(|phase| *phase == CatalogPublicationPhase::ReplaceHead)
        .ok_or_else(|| "catalog order omits head replacement".into())
}

/// Publishes one catalog-only candidate through `storage`.
fn publish(
    storage: &mut ModelStorage,
    call: &CandidateCall<'_, '_>,
) -> Result<RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError> {
    publish_retained_catalog_generation(
        storage,
        call.expectation,
        SegmentPublication::none(),
        call.catalog,
        call.segments,
    )
}

#[test]
fn preserving_successor_publishes_and_binds_the_preserved_manifest() -> Result<(), Box<dyn Error>> {
    let records = RetentionRecords::golden()?;
    let head = records.current_head()?;
    let entries = records.entries()?;
    let mut storage = ModelStorage::new(records);

    let (receipt, digest) = with_candidate(Candidate::Preserving, |call| {
        (
            publish(&mut storage, &call),
            call.catalog.checksummed().digest(),
        )
    })?;
    let receipt = receipt?;
    let preserved = receipt
        .preserved_manifest()
        .ok_or("receipt omits retention")?;
    let entry = *entries.first().ok_or("golden manifest has no entry")?;

    assert_eq!(receipt.outcome(), CatalogPublicationOutcome::Published);
    assert_eq!(receipt.generation(), CatalogGeneration::new(2)?);
    assert_eq!(receipt.catalog_digest(), digest);
    assert_eq!(preserved.generation(), head.generation());
    assert_eq!(preserved.digest(), head.manifest_digest());
    assert_eq!(preserved.root_count(), 1);
    assert_eq!(storage.stages(), catalog_only_phases());
    assert_eq!(
        storage.reads(),
        [
            (EvidenceRead::Head, 0),
            (EvidenceRead::Manifest, 0),
            (EvidenceRead::Root(entry), 0),
            (EvidenceRead::Head, stages_before_replacement()?),
        ]
    );
    Ok(())
}

#[test]
fn unretained_store_publishes_with_no_preserved_manifest() -> Result<(), Box<dyn Error>> {
    let mut storage = ModelStorage::new(RetentionRecords::unretained());

    let receipt = with_candidate(Candidate::Empty, |call| publish(&mut storage, &call))??;

    assert_eq!(receipt.outcome(), CatalogPublicationOutcome::Published);
    assert_eq!(receipt.preserved_manifest(), None);
    assert_eq!(
        storage.reads(),
        [
            (EvidenceRead::Head, 0),
            (EvidenceRead::Head, stages_before_replacement()?),
        ]
    );
    Ok(())
}

#[test]
fn already_published_candidate_proves_retention_before_reverification() -> Result<(), Box<dyn Error>>
{
    let records = RetentionRecords::golden()?;
    let mut storage = ModelStorage::with_stages(records, RecordingStorage::already_published());

    let receipt = with_candidate(Candidate::Preserving, |call| publish(&mut storage, &call))??;

    assert_eq!(
        receipt.outcome(),
        CatalogPublicationOutcome::AlreadyPublished
    );
    assert_eq!(
        receipt
            .preserved_manifest()
            .map(keep::PreservedRetentionManifest::root_count),
        Some(1)
    );
    assert_eq!(
        storage.stages(),
        [
            CatalogPublicationPhase::VerifyCurrent,
            CatalogPublicationPhase::SynchronizeRoot
        ]
    );
    assert!(storage.reads().iter().all(|(_, stages)| *stages == 0));
    Ok(())
}

#[test]
fn catalog_preflight_refuses_before_any_retention_read() -> Result<(), Box<dyn Error>> {
    let mut storage = ModelStorage::new(RetentionRecords::golden()?);

    let error = with_candidate(Candidate::Preserving, |call| {
        publish_retained_catalog_generation(
            &mut storage,
            keep::CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            call.catalog,
            call.segments,
        )
    })?;
    let error = require_error(error, "non-initial candidate published over no catalog")?;

    assert!(matches!(
        error,
        RetainedCatalogPublicationError::Catalog {
            source: CatalogPublicationError::InitialGeneration { observed }
        } if observed.get() == 2
    ));
    assert!(storage.reads().is_empty());
    assert!(storage.stages().is_empty());
    Ok(())
}

#[test]
fn every_catalog_fault_after_admission_keeps_its_exact_phase() -> Result<(), Box<dyn Error>> {
    let replacement = stages_before_replacement()?;
    for (index, failing) in catalog_only_phases().into_iter().enumerate() {
        let stages = RecordingStorage::failing_at(failing);
        let mut storage = ModelStorage::with_stages(RetentionRecords::golden()?, stages);

        let result = with_candidate(Candidate::Preserving, |call| publish(&mut storage, &call))?;
        let error = require_error(result, "faulting publication returned a receipt")?;
        let revalidated = storage.reads().len() == 4;

        assert!(matches!(
            error,
            RetainedCatalogPublicationError::Catalog {
                source: CatalogPublicationError::Storage { phase, .. }
            } if phase == failing
        ));
        assert_eq!(storage.stages().last(), Some(&failing));
        assert_eq!(revalidated, index >= replacement, "{failing:?}");
    }
    Ok(())
}
