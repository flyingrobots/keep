//! This module owns real version-one publication followed by migration and retention.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;

use keep::{
    AdmittedLayout, AdmittedRetentionRoot, AdmittedSegment, AdmittedSegmentRecord, BlobId,
    CanonicalCatalog, CanonicalLayoutRecord, CanonicalRetentionRoot, CatalogGeneration,
    CatalogPublicationExpectation, CatalogRestartByteLimit, CatalogRestartPolicy, ChunkSpan,
    FastCdc, FilesystemCatalogPublisher, FilesystemCatalogSnapshot, FilesystemPlatformAdmission,
    FilesystemRetentionPublicationAuthority, FilesystemStoreMigrationAuthority,
    FilesystemVersionTwoAdmission, LayoutEntryLimit, RegisteredRetentionProfile,
    RegisteredStorageProfile, RetentionAnchor, RetentionClosureLimits,
    RetentionGenerationExpectation, RetentionNamespace, RetentionPolicy, RetentionRoot,
    RootGeneration, SegmentReadPolicy, SegmentRecordLimit, StagedSegment,
    execute_retention_publication, execute_store_migration, preflight_retention_transition,
    prepare_retention_publication, publish_catalog_generation,
};

use super::durable_sandbox::TestDirectory;

type TestResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Records {
    Complete,
    LayoutsOnly,
}

pub(super) struct Identified {
    pub(super) spans: Vec<ChunkSpan>,
    pub(super) record: CanonicalLayoutRecord,
    pub(super) target: BlobId,
}

pub(super) fn identify(bytes: &[u8]) -> TestResult<Identified> {
    let target = BlobId::hash_bytes(bytes)?;
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    let layout = AdmittedLayout::from_spans(
        target,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        spans.clone(),
        LayoutEntryLimit::MAXIMUM,
    )?;
    Ok(Identified {
        spans,
        record: layout.encode_record()?,
        target,
    })
}

pub(super) fn policy() -> TestResult<CatalogRestartPolicy> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(16_777_216)?,
    ))
}

pub(super) fn build(name: &str, contents: &[&[u8]]) -> TestResult<TestDirectory> {
    let sandbox = TestDirectory::create(name)?;
    let identified = contents
        .iter()
        .map(|bytes| identify(bytes))
        .collect::<TestResult<Vec<_>>>()?;
    publish_catalog(&sandbox, contents, &identified, Records::Complete)?;
    migrate(&sandbox)?;
    publish_retention(&sandbox, &identified)?;
    Ok(sandbox)
}

pub(super) fn build_missing_chunk(name: &str, content: &[u8]) -> TestResult<TestDirectory> {
    let sandbox = TestDirectory::create(name)?;
    let identified = [identify(content)?];
    publish_catalog(&sandbox, &[content], &identified, Records::LayoutsOnly)?;
    migrate(&sandbox)?;
    Ok(sandbox)
}

fn migrate(sandbox: &TestDirectory) -> TestResult<()> {
    let admission = FilesystemPlatformAdmission::reopen(sandbox.path())?;
    let mut migration = FilesystemStoreMigrationAuthority::open(
        admission,
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
    )?;
    let intent = migration.observe_intent()?;
    let _receipt = execute_store_migration(&mut migration, &intent)?;
    drop(migration);
    Ok(())
}

fn publish_catalog(
    sandbox: &TestDirectory,
    contents: &[&[u8]],
    identified: &[Identified],
    records: Records,
) -> TestResult<()> {
    let admission = FilesystemPlatformAdmission::initialize(sandbox.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(admission, policy()?)?;
    let mut stage = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?;
    let mut chunks = BTreeSet::new();
    let mut layouts = BTreeSet::new();
    for (bytes, identity) in contents.iter().zip(identified) {
        for span in &identity.spans {
            if records == Records::Complete && chunks.insert(span.id()) {
                let start = usize::try_from(span.offset().get())?;
                let end = usize::try_from(span.end().get())?;
                stage = stage.append(AdmittedSegmentRecord::for_chunk(
                    bytes.get(start..end).ok_or("chunk outside source")?,
                )?)?;
            }
        }
        if layouts.insert(identity.record.id()) {
            stage = stage.append(AdmittedSegmentRecord::for_layout(&identity.record)?)?;
        }
    }
    let sealed = stage.seal()?;
    let bytes = fs::read(sandbox.path().join("staging/current.seg"))?;
    let segments = [AdmittedSegment::decode(
        &bytes,
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
    )?];
    let selected = publisher.select_segment(sealed, segments.first().ok_or("segment absent")?)?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let _receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selected,
        &catalog,
        &segments,
    )?;
    Ok(())
}

fn publish_retention(sandbox: &TestDirectory, identified: &[Identified]) -> TestResult<()> {
    let anchors = identified
        .iter()
        .map(|entry| RetentionAnchor::new(entry.target, entry.record.id()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(&b"worldline"[..])?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 16_777_216, 67_108_864)?,
        ),
        None,
        anchors,
    )?;
    let canonical = CanonicalRetentionRoot::from_root(&root)?;
    let candidate = AdmittedRetentionRoot::decode(canonical.encoded())?;
    let current = FilesystemCatalogSnapshot::load(sandbox.path(), policy()?)?;
    let snapshot = current.snapshot()?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )?;
    let prepared = prepare_retention_publication(preflight, None)?;
    let admission = FilesystemVersionTwoAdmission::reopen(sandbox.path())?;
    let mut authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    let _receipt = execute_retention_publication(&mut authority, &prepared)?;
    Ok(())
}
