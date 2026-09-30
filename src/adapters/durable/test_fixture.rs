//! A migrated version-two store holding several anchored blobs, one of them
//! spanning many chunks, and one committed layout no root anchors.

use std::error::Error;
use std::fs;
use std::path::Path;

use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store, reopen_authority,
};
use crate::adapters::{
    AdmittedRetentionRoot, AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog,
    CanonicalRetentionRoot, CatalogPublicationExpectation, FilesystemCatalogPublisher,
    FilesystemCatalogSnapshot, FilesystemVersionTwoAdmission, SegmentRecordLimit, StagedSegment,
    publish_catalog_generation,
};
use crate::{
    AdmittedLayout, BlobHasher, BlobId, CanonicalLayoutRecord, CatalogGeneration, ChunkSpan,
    FastCdc, LayoutEntryLimit, LayoutId, RegisteredRetentionProfile, RegisteredStorageProfile,
    RetentionAnchor, RetentionClosureLimits, RetentionGenerationExpectation, RetentionNamespace,
    RetentionPolicy, RetentionRoot, RootGeneration, execute_retention_publication,
    preflight_retention_transition, prepare_retention_publication,
};

/// A built store with each content's identities, in the given order.
pub(super) type BuiltStore = (TestDirectory, Vec<Published>);

/// One published blob's identities.
#[derive(Clone, Copy, Debug)]
pub(super) struct Published {
    pub(super) target: BlobId,
    pub(super) layout: LayoutId,
}

/// Deterministic pseudo-random bytes long enough to span several chunks.
pub(super) fn long_content() -> Vec<u8> {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    (0..512 * 1024)
        .map(|_| {
            state ^= state.wrapping_shl(13);
            state ^= state.wrapping_shr(7);
            state ^= state.wrapping_shl(17);
            state.to_le_bytes().first().copied().unwrap_or_default()
        })
        .collect()
}

struct Identified {
    target: BlobId,
    spans: Vec<ChunkSpan>,
    record: CanonicalLayoutRecord,
}

fn identify(bytes: &[u8]) -> Result<Identified, Box<dyn Error>> {
    let mut hasher = BlobHasher::new();
    hasher.update(bytes)?;
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    let layout = AdmittedLayout::from_spans(
        hasher.finish(),
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        spans.clone(),
        LayoutEntryLimit::MAXIMUM,
    )?;
    Ok(Identified {
        target: layout.target(),
        spans,
        record: layout.encode_record()?,
    })
}

/// Publishes every content as chunk and layout records in one new segment
/// under catalog generation two, then anchors every content except the
/// last under retention generation one. Returns each content's identities
/// in the given order.
pub(super) fn durable_store(name: &str, contents: &[&[u8]]) -> Result<BuiltStore, Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let policy = catalog_policy()?;
    let mut bundle_bytes = None;
    for entry in fs::read_dir(sandbox.path().join("segments"))? {
        bundle_bytes = Some(fs::read(entry?.path())?);
    }
    let bundle_bytes = bundle_bytes.ok_or("the migrated store has no segment")?;
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;
    let mut publisher = FilesystemCatalogPublisher::open_version_two(admission, policy)?;
    let current = FilesystemCatalogSnapshot::load(sandbox.path(), policy)?;
    let snapshot = current.snapshot()?;
    let identified = contents
        .iter()
        .map(|bytes| identify(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let mut staged = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?;
    let mut identities = Vec::new();
    for (bytes, identity) in contents.iter().zip(&identified) {
        for span in &identity.spans {
            let start = usize::try_from(span.offset().get())?;
            let end = usize::try_from(span.end().get())?;
            let chunk = bytes.get(start..end).ok_or("span outside content")?;
            staged = staged.append(AdmittedSegmentRecord::for_chunk(chunk)?)?;
        }
        staged = staged.append(AdmittedSegmentRecord::for_layout(&identity.record)?)?;
        identities.push(Published {
            target: identity.target,
            layout: identity.record.id(),
        });
    }
    let sealed = staged.seal()?;
    let staged_bytes = fs::read(sandbox.path().join("staging").join("current.seg"))?;
    let bundle = AdmittedSegment::decode(&bundle_bytes, policy.segment_read())?;
    let second = AdmittedSegment::decode(&staged_bytes, policy.segment_read())?;
    let segments = [bundle, second];
    let staged_segment = segments.get(1).ok_or("staged segment")?;
    let selection = publisher.select_segment(sealed, staged_segment)?;
    let successor = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(snapshot.catalog_digest()),
        &segments,
    )?;
    let _receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::successor_of(&snapshot),
        selection,
        &successor,
        &segments,
    )?;
    drop(snapshot);
    drop(current);
    drop(publisher);
    let anchored: Vec<_> = identities
        .iter()
        .take(identities.len().saturating_sub(1))
        .map(|entry| RetentionAnchor::new(entry.target, entry.layout))
        .collect();
    publish_root(sandbox.path(), anchored)?;
    Ok((sandbox, identities))
}

fn publish_root(root: &Path, anchors: Vec<RetentionAnchor>) -> Result<(), Box<dyn Error>> {
    let retention_root = RetentionRoot::new(
        RetentionNamespace::try_from(vec![0x2f])?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 1 << 24, 1 << 26)?,
        ),
        None,
        anchors,
    )?;
    let root_bytes = CanonicalRetentionRoot::from_root(&retention_root)?
        .encoded()
        .to_vec();
    let policy = catalog_policy()?;
    let current = FilesystemCatalogSnapshot::load(root, policy)?;
    let snapshot = current.snapshot()?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )?;
    let preparation = prepare_retention_publication(preflight, None)?;
    let mut authority = reopen_authority(root)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    Ok(())
}
