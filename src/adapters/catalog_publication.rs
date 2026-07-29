//! Preflighted catalog-generation publication orchestration.

use super::catalog_publication_expectation::ExpectedCurrentCatalog;
use super::{
    AdmittedCatalog, AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead,
    CatalogPublicationError, CatalogPublicationExpectation, CatalogPublicationReadiness,
    CatalogPublicationReceipt, CatalogPublicationStorage, CatalogTransitionError,
    ChecksummedPublicationHead, SegmentPublication, catalog_publication_execution,
};

/// Publishes one fully admitted canonical catalog generation.
///
/// All catalog, segment, and head relationships are verified before the first
/// storage transition. A new publication returns only after atomic head
/// replacement and root-directory synchronization. If the complete candidate
/// is already current, the retry performs no publication mutation,
/// re-synchronizes the root, and returns an explicit already-published outcome.
///
/// # Errors
///
/// Returns [`CatalogPublicationError`] for a staged segment outside the
/// admitted set, catalog or head preflight refusal, snapshot disagreement, or
/// an exact storage transition failure. A failure returns no receipt.
pub fn publish_catalog_generation(
    storage: &mut impl CatalogPublicationStorage,
    expectation: CatalogPublicationExpectation,
    segment: SegmentPublication<'_, '_>,
    catalog: &CanonicalCatalog,
    segments: &[AdmittedSegment<'_>],
) -> Result<CatalogPublicationReceipt, CatalogPublicationError> {
    validate_staged_segment(&segment, segments)?;
    let checksummed = catalog.checksummed();
    let admitted = checksummed.admit(segments).map_err(|source| {
        CatalogPublicationError::CatalogAdmission {
            source: Box::new(source),
        }
    })?;
    validate_transition(expectation, &admitted)?;
    let head = CanonicalPublicationHead::for_catalog(checksummed);
    let checked_head = ChecksummedPublicationHead::decode(head.encoded())
        .map_err(|source| CatalogPublicationError::HeadVerification { source })?;
    let snapshot = checked_head
        .admit(admitted)
        .map_err(|source| CatalogPublicationError::SnapshotAdmission { source })?;
    let readiness =
        catalog_publication_execution::execute_current(storage, expectation, &snapshot, &segment)?;
    if readiness == CatalogPublicationReadiness::AlreadyPublished {
        return Ok(CatalogPublicationReceipt::already_published(
            snapshot.generation(),
            snapshot.catalog_digest(),
        ));
    }
    if let Some(segment) = segment.into_admitted() {
        catalog_publication_execution::execute_segment(storage, segment)?;
    }
    catalog_publication_execution::execute_catalog(storage, catalog, checksummed)?;
    catalog_publication_execution::execute_head(storage, &head, &snapshot)?;
    Ok(CatalogPublicationReceipt::published(
        snapshot.generation(),
        snapshot.catalog_digest(),
    ))
}

fn validate_transition(
    expectation: CatalogPublicationExpectation,
    candidate: &AdmittedCatalog<'_, '_>,
) -> Result<(), CatalogPublicationError> {
    match expectation.current() {
        ExpectedCurrentCatalog::Uninitialized => validate_initial(candidate),
        ExpectedCurrentCatalog::Published { generation, digest } => {
            let expected =
                generation
                    .successor()
                    .map_err(|source| CatalogPublicationError::Transition {
                        source: CatalogTransitionError::GenerationExhausted { source },
                    })?;
            if candidate.generation() != expected {
                return Err(CatalogPublicationError::Transition {
                    source: CatalogTransitionError::Generation {
                        expected,
                        observed: candidate.generation(),
                    },
                });
            }
            if candidate.previous_catalog_digest() != Some(digest) {
                return Err(CatalogPublicationError::Transition {
                    source: CatalogTransitionError::Predecessor {
                        expected: digest,
                        observed: candidate.previous_catalog_digest(),
                    },
                });
            }
            Ok(())
        }
    }
}

const fn validate_initial(
    candidate: &AdmittedCatalog<'_, '_>,
) -> Result<(), CatalogPublicationError> {
    if candidate.generation().get() != 1 {
        return Err(CatalogPublicationError::InitialGeneration {
            observed: candidate.generation(),
        });
    }
    Ok(())
}

fn validate_staged_segment(
    selection: &SegmentPublication<'_, '_>,
    segments: &[AdmittedSegment<'_>],
) -> Result<(), CatalogPublicationError> {
    let Some(staged) = selection.admitted() else {
        return Ok(());
    };
    let digest = staged.digest();
    if segments
        .iter()
        .any(|candidate| candidate.digest() == digest)
    {
        Ok(())
    } else {
        Err(CatalogPublicationError::StagedSegmentNotAdmitted {
            segment_digest: digest,
        })
    }
}
