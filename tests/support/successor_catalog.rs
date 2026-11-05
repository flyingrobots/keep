//! This module owns test publication of catalogs after a store's first one.
//!
//! Every successor goes through `FilesystemCatalogPublisher` and the
//! retained-closure gate, on a store created by `populated_store.rs`. Callers
//! pass the publisher so they can observe or interrupt its transitions.
//!
//! Consumers include this file by path beside `populated_store`.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CatalogGeneration,
    CatalogPublicationExpectation, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemSegmentStage, RetainedCatalogPublicationError, RetainedCatalogPublicationReceipt,
    SealedSegment, SegmentPublication, SegmentRecordLimit, StagedSegment,
    publish_retained_catalog_generation,
};

use super::populated_store::{
    FirstCatalogRecords, Identified, StoreAdmission, TestResult, identify, policy, segment_policy,
    select_records,
};

/// The gate's exact outcome, inside the fixture's own failure channel.
pub(super) type GateOutcome =
    Result<RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError>;

/// Whether a candidate catalog keeps every predecessor segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Predecessor {
    /// The candidate references every current segment and the new one.
    Kept,
    /// The candidate references only the new segment.
    Dropped,
}

/// Opens the writer over a created store.
pub(super) fn open_publisher(
    root: &Path,
    admission: StoreAdmission,
) -> TestResult<FilesystemCatalogPublisher> {
    Ok(FilesystemCatalogPublisher::open(
        (admission.reopen)(root)?,
        policy()?,
    )?)
}

/// Reads every immutable segment in deterministic name order.
pub(super) fn pool_segments(root: &Path) -> TestResult<Vec<Vec<u8>>> {
    let mut names = fs::read_dir(root.join("segments"))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    names
        .iter()
        .map(|name| Ok(fs::read(root.join("segments").join(name))?))
        .collect()
}

pub(super) fn admit_all(encoded: &[Vec<u8>]) -> TestResult<Vec<AdmittedSegment<'_>>> {
    encoded
        .iter()
        .map(|bytes| Ok(AdmittedSegment::decode(bytes, segment_policy())?))
        .collect()
}

/// Every distinct chunk record of `content` followed by its layout record.
pub(super) fn blob_records<'content>(
    content: &'content [u8],
    identified: &'content Identified,
) -> TestResult<Vec<AdmittedSegmentRecord<'content>>> {
    select_records([(content, identified)], FirstCatalogRecords::Complete)
}

/// Stages and seals every record of `content` without selecting the stage.
pub(super) fn stage_blob<'publisher>(
    publisher: &'publisher FilesystemCatalogPublisher,
    content: &[u8],
    identified: &Identified,
) -> TestResult<SealedSegment<FilesystemSegmentStage<'publisher>>> {
    let mut stage = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?;
    for record in blob_records(content, identified)? {
        stage = stage.append(record)?;
    }
    Ok(stage.seal()?)
}

/// Stages `content` as one new segment and publishes the next catalog generation.
///
/// The candidate references the current segments when `predecessor` is
/// [`Predecessor::Kept`].
pub(super) fn publish(
    publisher: &mut FilesystemCatalogPublisher,
    root: &Path,
    content: &[u8],
    predecessor: Predecessor,
) -> TestResult<GateOutcome> {
    let identified = identify(content)?;
    let current = FilesystemCatalogSnapshot::load(root, policy()?)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current.snapshot()?);
    let mut encoded = match predecessor {
        Predecessor::Kept => pool_segments(root)?,
        Predecessor::Dropped => Vec::new(),
    };
    let sealed = stage_blob(publisher, content, &identified)?;
    encoded.push(fs::read(root.join("staging/current.seg"))?);
    let segments = admit_all(&encoded)?;
    let staged = segments.last().ok_or("staged segment absent")?;
    let selection = publisher.select_segment(sealed, staged)?;
    let catalog = successor(&current, &segments)?;
    Ok(publish_retained_catalog_generation(
        publisher,
        expectation,
        selection,
        &catalog,
        &segments,
    ))
}

/// Publishes the next catalog generation over already pooled segments.
pub(super) fn republish(
    publisher: &mut FilesystemCatalogPublisher,
    root: &Path,
) -> TestResult<GateOutcome> {
    let current = FilesystemCatalogSnapshot::load(root, policy()?)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current.snapshot()?);
    let encoded = pool_segments(root)?;
    let segments = admit_all(&encoded)?;
    let catalog = successor(&current, &segments)?;
    Ok(publish_retained_catalog_generation(
        publisher,
        expectation,
        SegmentPublication::none(),
        &catalog,
        &segments,
    ))
}

/// The catalog one generation after `current` over exactly `segments`.
fn successor(
    current: &FilesystemCatalogSnapshot,
    segments: &[AdmittedSegment<'_>],
) -> TestResult<CanonicalCatalog> {
    let generation = CatalogGeneration::new(
        current
            .generation()
            .get()
            .checked_add(1)
            .ok_or("generation overflow")?,
    )?;
    Ok(CanonicalCatalog::from_segments(
        generation,
        Some(current.catalog_digest()),
        segments,
    )?)
}
