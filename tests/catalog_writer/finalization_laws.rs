//! This module owns gated recovery finalization of a staged catalog head.
//!
//! Each law stages exactly the residue an interrupted publication leaves after
//! `head.next` synchronization: the new segment and candidate catalog in their
//! immutable pools and a complete `head.next`. Recovery then decides.

use std::fs;
use std::path::Path;

use keep::{
    CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration, ChecksummedPublicationHead,
    FilesystemCatalogSnapshot, RecoveryEntryRole, RecoveryNextHeadFinalizationError,
    RecoveryNextHeadFinalizationOutcome, RecoveryNextHeadFinalizationStorageError,
    RetainedCatalogPublicationError,
};

use super::durable_fixture;
use super::recovery_fixture::{catalog_name, finalize, stage_roles};
use super::writer_fixture::{
    Predecessor, TestResult, admit_all, assert_reads, first_blob, open_publisher, pool_segments,
    second_blob, witness,
};

/// Pools a new segment for `content` and stages `head.next` over a candidate.
///
/// Returns the exact segment bytes the staged candidate references.
fn stage_next_head(
    root: &Path,
    content: &[u8],
    predecessor: Predecessor,
) -> TestResult<Vec<Vec<u8>>> {
    let previous = pool_segments(root)?;
    let publisher = open_publisher(root)?;
    let staged = super::writer_fixture::stage_only(&publisher, root, content)?;
    drop(publisher);
    let segment = admit_all(std::slice::from_ref(&staged))?;
    let name = super::writer_fixture::segment_name(segment.first().ok_or("segment absent")?)?;
    fs::rename(
        root.join("staging/current.seg"),
        root.join("segments").join(name),
    )?;
    let mut candidate = match predecessor {
        Predecessor::Kept => previous,
        Predecessor::Dropped => Vec::new(),
    };
    candidate.push(staged);
    let segments = admit_all(&candidate)?;
    let current = FilesystemCatalogSnapshot::load(root, durable_fixture::policy()?)?;
    let catalog = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(current.catalog_digest()),
        &segments,
    )?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let checked = ChecksummedPublicationHead::decode(head.encoded())?;
    fs::write(
        root.join("catalogs").join(catalog_name(&checked)?),
        catalog.encoded(),
    )?;
    fs::write(root.join("head.next"), head.encoded())?;
    Ok(candidate)
}

#[test]
fn recovery_finalizes_a_staged_head_that_preserves_retained_content() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-finalize-preserving", &[&first])?;
    let root = sandbox.path();
    let candidate = stage_next_head(root, &second_blob(), Predecessor::Kept)?;
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::NextHeadStage]);
    // A retained head stage does not stop readers: they select only HEAD.
    assert_reads(root, &first)?;

    let receipt = finalize(root, &candidate)??;
    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(receipt.target().generation().get(), 2);
    assert!(stage_roles(root)?.is_empty());
    assert_reads(root, &first)
}

#[test]
fn recovery_refuses_a_staged_head_that_would_lose_retained_content() -> TestResult<()> {
    let first = first_blob();
    let sandbox = durable_fixture::build("v2-finalize-dropping", &[&first])?;
    let root = sandbox.path();
    let candidate = stage_next_head(root, &second_blob(), Predecessor::Dropped)?;
    let before = witness(root)?;

    let outcome = finalize(root, &candidate)?;
    let Err(RecoveryNextHeadFinalizationError::Verify { source, .. }) = outcome else {
        return Err(
            format!("a head that loses retained content was finalized: {outcome:?}").into(),
        );
    };
    let RecoveryNextHeadFinalizationStorageError::RetainedClosure { source } = *source else {
        return Err(format!("finalization refused for another reason: {source:?}").into());
    };
    assert!(matches!(
        *source,
        RetainedCatalogPublicationError::Closure { .. }
    ));
    assert_eq!(
        witness(root)?,
        before,
        "refusal must leave its evidence intact"
    );
    assert_eq!(stage_roles(root)?, [RecoveryEntryRole::NextHeadStage]);
    assert_reads(root, &first)
}
