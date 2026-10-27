//! This module owns catalog publication that refuses to lose retained content.
//!
//! A store with retention state publishes a candidate catalog only through
//! [`publish_retained_catalog_generation`]. The entry point admits the current
//! global retention head, its manifest, and every root that manifest names,
//! then verifies each root's closure against the candidate before the first
//! catalog mutation. It confirms the retention head is unchanged again
//! immediately before `HEAD` replacement. The decision and its crash states are
//! recorded in `docs/formats/segment-store-v2/rationale.md`.

mod preserved_retention_manifest;
mod retained_catalog_publication_error;
mod retained_catalog_publication_error_display;
mod retained_catalog_publication_receipt;
mod retained_catalog_publication_storage;
mod retained_closure_admission;
mod retained_evidence_admission;
mod retention_evidence_phase;

pub use preserved_retention_manifest::PreservedRetentionManifest;
pub use retained_catalog_publication_error::RetainedCatalogPublicationError;
pub use retained_catalog_publication_receipt::RetainedCatalogPublicationReceipt;
pub use retained_catalog_publication_storage::RetainedCatalogPublicationStorage;
pub use retained_closure_admission::RetainedClosureAdmission;
pub use retention_evidence_phase::RetentionEvidencePhase;

use super::catalog_publication_execution as execution;
use super::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationReadiness, CatalogPublicationReceipt, CatalogSnapshot, SegmentPublication,
    catalog_publication,
};

/// Publishes one catalog generation only if it preserves every retained closure.
///
/// The caller's storage must hold exclusive writer authority for the complete
/// call. The order is:
///
/// 1. admit the candidate catalog, its successor relationship, and its
///    generated next head without I/O, exactly as
///    [`publish_catalog_generation`](crate::publish_catalog_generation) does;
/// 2. read and admit the current retention head, the manifest it names, and
///    every root that manifest names, in manifest order;
/// 3. verify each root's closure against the candidate with the root's own
///    registered profile and admitted limits;
/// 4. run the catalog publication protocol through `head.next` verification;
/// 5. reread the retention head and require it to equal the admitted head; and
/// 6. replace `HEAD` and synchronize the store root.
///
/// Steps 2 and 3 complete before the first catalog storage call, so a refused
/// candidate leaves no stage, pool entry, or head change. Memory is bounded by
/// one manifest and one root at a time; closure work is bounded by each root's
/// limits and the manifest's 4,096-entry ceiling. The first refusal in
/// manifest order is returned.
///
/// # Errors
///
/// Returns [`RetainedCatalogPublicationError`] with the exact catalog preflight
/// or storage failure, retention read failure, retention record refusal,
/// manifest or root binding disagreement, closure refusal and the manifest
/// entry it belongs to, or a retention head that changed before commit. A
/// failure returns no receipt.
pub fn publish_retained_catalog_generation<S: RetainedCatalogPublicationStorage>(
    storage: &mut S,
    expectation: CatalogPublicationExpectation,
    segment: SegmentPublication<'_, '_>,
    catalog: &CanonicalCatalog,
    segments: &[AdmittedSegment<'_>],
) -> Result<RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError> {
    let checksummed = catalog.checksummed();
    let admitted =
        catalog_publication::admit_candidate(expectation, &segment, checksummed, segments)?;
    let head = CanonicalPublicationHead::for_catalog(checksummed);
    let snapshot = catalog_publication::admit_snapshot(&head, admitted)?;
    let admission = retained_evidence_admission::admit(storage, &snapshot)?;
    let preserved = admission.preserved_manifest();
    let current_stages = storage.catalog_stages(&admission);
    let readiness = execution::execute_current(current_stages, expectation, &snapshot, &segment)?;
    if readiness == CatalogPublicationReadiness::AlreadyPublished {
        let catalog_receipt = CatalogPublicationReceipt::already_published(
            snapshot.generation(),
            snapshot.catalog_digest(),
        );
        return Ok(RetainedCatalogPublicationReceipt::new(
            catalog_receipt,
            preserved,
        ));
    }
    let candidate = StagedCandidate {
        catalog,
        head: &head,
        snapshot: &snapshot,
    };
    stage_candidate(storage.catalog_stages(&admission), segment, &candidate)?;
    retained_evidence_admission::revalidate(storage, &admission)?;
    execution::execute_head_replacement(storage.catalog_stages(&admission))?;
    let catalog_receipt =
        CatalogPublicationReceipt::published(snapshot.generation(), snapshot.catalog_digest());
    Ok(RetainedCatalogPublicationReceipt::new(
        catalog_receipt,
        preserved,
    ))
}

struct StagedCandidate<'call, 'head, 'catalog, 'records> {
    catalog: &'call CanonicalCatalog,
    head: &'call CanonicalPublicationHead,
    snapshot: &'call CatalogSnapshot<'head, 'catalog, 'records>,
}

fn stage_candidate(
    stages: &mut impl super::CatalogPublicationStorage,
    segment: SegmentPublication<'_, '_>,
    candidate: &StagedCandidate<'_, '_, '_, '_>,
) -> Result<(), RetainedCatalogPublicationError> {
    if let Some(admitted) = segment.into_admitted() {
        execution::execute_segment(stages, admitted)?;
    }
    execution::execute_catalog(stages, candidate.catalog, candidate.catalog.checksummed())?;
    execution::execute_head_stage(stages, candidate.head, candidate.snapshot)?;
    Ok(())
}
