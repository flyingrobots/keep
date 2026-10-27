//! This module owns one blob's ingestion through the version-two catalog writer.
//!
//! The library's interruption laws include this file by path, so it names the
//! crate as `keep` and reaches the shared store fixture through `super`.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, AdmittedSegment, AdmittedSegmentRecord,
    CanonicalCatalog, CanonicalRetentionRoot, CatalogGeneration, CatalogPublicationExpectation,
    DurableStore, FilesystemCatalogSnapshot, FilesystemRetentionPublicationAuthority,
    FilesystemVersionTwoAdmission, FilesystemVersionTwoCatalogPublisher, LayoutEntryLimit,
    ReaderAttemptLimit, RegisteredRetentionProfile, RetainedCatalogPublicationError,
    RetainedCatalogPublicationReceipt, RetentionAnchor, RetentionClosureLimits,
    RetentionGenerationExpectation, RetentionNamespace, RetentionPolicy, RetentionRoot,
    RootGeneration, SegmentPublication, SegmentReadPolicy, SegmentRecordLimit, StagedSegment,
    execute_retention_publication, preflight_retention_transition, prepare_retention_publication,
    publish_retained_catalog_generation,
};

use super::durable_fixture::{self, Identified};

pub(super) type TestResult<T> = Result<T, Box<dyn Error>>;

/// The retained content every migrated fixture store starts with.
pub(super) fn first_blob() -> Vec<u8> {
    b"keep version-two writer: first retained blob. ".repeat(3_000)
}

/// The content each law ingests through the version-two writer.
pub(super) fn second_blob() -> Vec<u8> {
    b"KEEP V2 WRITER -- the second blob, ingested after migration | ".repeat(2_500)
}

/// Whether a candidate catalog keeps every predecessor segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Predecessor {
    /// The candidate references every current segment and the new one.
    Kept,
    /// The candidate references only the new segment.
    Dropped,
}

pub(super) const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

/// Opens the version-two writer over a migrated store.
pub(super) fn open_publisher(root: &Path) -> TestResult<FilesystemVersionTwoCatalogPublisher> {
    let admission = FilesystemVersionTwoAdmission::reopen(root)?;
    Ok(FilesystemVersionTwoCatalogPublisher::open(
        admission,
        durable_fixture::policy()?,
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

/// Stages `content` as one new segment and publishes the next catalog generation.
///
/// The candidate references the current segments when `predecessor` is
/// [`Predecessor::Kept`]. The outer result carries fixture failures; the inner
/// result is the gate's exact outcome.
pub(super) fn publish(
    publisher: &mut FilesystemVersionTwoCatalogPublisher,
    root: &Path,
    content: &[u8],
    predecessor: Predecessor,
) -> TestResult<Result<RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError>> {
    let identified = durable_fixture::identify(content)?;
    let current = FilesystemCatalogSnapshot::load(root, durable_fixture::policy()?)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current.snapshot()?);
    let mut encoded = match predecessor {
        Predecessor::Kept => pool_segments(root)?,
        Predecessor::Dropped => Vec::new(),
    };
    let sealed = stage_records(publisher, content, &identified)?;
    encoded.push(fs::read(root.join("staging/current.seg"))?);
    let segments = admit_all(&encoded)?;
    let staged = segments.last().ok_or("staged segment absent")?;
    let selection = publisher.select_segment(sealed, staged)?;
    let generation = CatalogGeneration::new(
        current
            .generation()
            .get()
            .checked_add(1)
            .ok_or("generation overflow")?,
    )?;
    let catalog =
        CanonicalCatalog::from_segments(generation, Some(current.catalog_digest()), &segments)?;
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
    publisher: &mut FilesystemVersionTwoCatalogPublisher,
    root: &Path,
) -> TestResult<Result<RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError>> {
    let current = FilesystemCatalogSnapshot::load(root, durable_fixture::policy()?)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current.snapshot()?);
    let encoded = pool_segments(root)?;
    let segments = admit_all(&encoded)?;
    let generation = CatalogGeneration::new(
        current
            .generation()
            .get()
            .checked_add(1)
            .ok_or("generation overflow")?,
    )?;
    let catalog =
        CanonicalCatalog::from_segments(generation, Some(current.catalog_digest()), &segments)?;
    Ok(publish_retained_catalog_generation(
        publisher,
        expectation,
        SegmentPublication::none(),
        &catalog,
        &segments,
    ))
}

pub(super) fn admit_all(encoded: &[Vec<u8>]) -> TestResult<Vec<AdmittedSegment<'_>>> {
    encoded
        .iter()
        .map(|bytes| Ok(AdmittedSegment::decode(bytes, segment_policy())?))
        .collect()
}

fn stage_records<'publisher>(
    publisher: &'publisher FilesystemVersionTwoCatalogPublisher,
    content: &[u8],
    identified: &Identified,
) -> TestResult<keep::SealedSegment<keep::FilesystemSegmentStage<'publisher>>> {
    let mut stage = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?;
    let mut chunks = BTreeSet::new();
    for span in &identified.spans {
        if chunks.insert(span.id()) {
            let start = usize::try_from(span.offset().get())?;
            let end = usize::try_from(span.end().get())?;
            let bytes = content.get(start..end).ok_or("chunk outside source")?;
            stage = stage.append(AdmittedSegmentRecord::for_chunk(bytes)?)?;
        }
    }
    stage = stage.append(AdmittedSegmentRecord::for_layout(&identified.record)?)?;
    Ok(stage.seal()?)
}

/// Publishes a new generation-one retention root that anchors `content`.
pub(super) fn retain(root: &Path, namespace: &[u8], content: &[u8]) -> TestResult<()> {
    let identified = durable_fixture::identify(content)?;
    let retained = RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 16_777_216, 67_108_864)?,
        ),
        None,
        vec![RetentionAnchor::new(
            identified.target,
            identified.record.id(),
        )],
    )?;
    let canonical = CanonicalRetentionRoot::from_root(&retained)?;
    let candidate = AdmittedRetentionRoot::decode(canonical.encoded())?;
    let current = FilesystemCatalogSnapshot::load(root, durable_fixture::policy()?)?;
    let snapshot = current.snapshot()?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )?;
    let admission = FilesystemVersionTwoAdmission::reopen(root)?;
    let mut authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    let observed = authority.observe_current()?;
    let manifest = observed
        .as_ref()
        .map(|state| AdmittedRetentionManifest::decode(state.manifest_bytes()))
        .transpose()?;
    let prepared = prepare_retention_publication(preflight, manifest.as_ref())?;
    let _receipt = execute_retention_publication(&mut authority, &prepared)?;
    Ok(())
}

/// Requires a fresh fenced view to reconstruct exactly `content`.
pub(super) fn assert_reads(root: &Path, content: &[u8]) -> TestResult<()> {
    let target = durable_fixture::identify(content)?.target;
    let store = DurableStore::open(
        root,
        durable_fixture::policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let mut output = Vec::new();
    let _receipt = store.reconstruct(target, &mut output)?;
    if output == content {
        Ok(())
    } else {
        Err("reconstructed bytes differ from the ingested content".into())
    }
}

/// Every store path, with the bytes of each regular file, in path order.
pub(super) type StoreWitness = Vec<(String, Option<Vec<u8>>)>;

/// Every path and regular-file byte string under `root`, in name order.
pub(super) fn witness(root: &Path) -> TestResult<StoreWitness> {
    let mut entries = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            let relative = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF-8 store path")?
                .to_owned();
            if path.is_dir() {
                pending.push(path);
                entries.push((relative, None));
            } else {
                entries.push((relative, Some(fs::read(&path)?)));
            }
        }
    }
    entries.sort();
    Ok(entries)
}

/// Stages and seals `content` as `staging/current.seg` without publishing it.
///
/// Returns the exact sealed bytes. The closed stage remains for the caller.
pub(super) fn stage_only(
    publisher: &FilesystemVersionTwoCatalogPublisher,
    root: &Path,
    content: &[u8],
) -> TestResult<Vec<u8>> {
    let identified = durable_fixture::identify(content)?;
    let sealed = stage_records(publisher, content, &identified)?;
    drop(sealed);
    Ok(fs::read(root.join("staging/current.seg"))?)
}

/// The canonical segment-pool name of one admitted segment.
pub(super) fn segment_name(segment: &AdmittedSegment<'_>) -> TestResult<String> {
    let mut name = String::new();
    for byte in segment.digest().as_bytes() {
        write!(name, "{byte:02x}")?;
    }
    name.push_str(".seg");
    Ok(name)
}
