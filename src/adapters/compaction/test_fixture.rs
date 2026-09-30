//! Shared fixtures for the compaction laws: a migrated store holding one
//! mixed segment, and the logical view compaction must leave untouched.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use super::{
    CompactionObservation, CompactionPlan, FilesystemCompactionAuthority, observe_compaction,
    plan_compaction,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::gc::{GcLimits, GcPlan, observe_gc_liveness, plan_gc};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store, reopen_authority,
};
use crate::adapters::{
    AdmittedRetentionRoot, AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog,
    CanonicalRetentionRoot, CatalogPublicationExpectation, FilesystemCatalogPublisher,
    FilesystemCatalogSnapshot, FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission,
    ReaderAttemptLimit, SegmentRecordIdentity, SegmentRecordLimit, StagedSegment,
    publish_catalog_generation,
};
use crate::{
    AdmittedLayout, BlobHasher, BlobId, CanonicalLayoutRecord, CatalogGeneration, FastCdc,
    LayoutEntryLimit, LayoutId, RegisteredRetentionProfile, RegisteredStorageProfile,
    RetentionAnchor, RetentionClosureLimits, RetentionGenerationExpectation, RetentionNamespace,
    RetentionPolicy, RetentionRoot, RootGeneration, execute_retention_publication,
    preflight_retention_transition, prepare_retention_publication,
};

/// Every retained closure's root digest and member identities, and every
/// live record's exact bytes.
pub(super) type LogicalView = (Closures, BTreeMap<SegmentRecordIdentity, Vec<u8>>);

pub(super) fn identify(
    bytes: &[u8],
) -> Result<(BlobId, LayoutId, CanonicalLayoutRecord), Box<dyn Error>> {
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
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let record = layout.encode_record()?;
    Ok((layout.target(), record.id(), record))
}

pub(super) fn pool_segment_bytes(root: &Path) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    for entry in fs::read_dir(root.join("segments"))? {
        bytes.push(fs::read(entry?.path())?);
    }
    Ok(bytes)
}

/// A migrated store whose second catalog generation adds a segment holding
/// blob `[1]`'s chunk and layout beside an unanchored chunk `[2]`, with
/// retention generation one anchoring blobs `[0]` and `[1]`.
pub(super) fn mixed_store(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let policy = catalog_policy()?;
    let bundle_bytes = pool_segment_bytes(sandbox.path())?
        .pop()
        .ok_or("the migrated store has no segment")?;
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;
    let mut publisher = FilesystemCatalogPublisher::open_version_two(admission, policy)?;
    let current = FilesystemCatalogSnapshot::load(sandbox.path(), policy)?;
    let snapshot = current.snapshot()?;
    let (_, _, layout_one) = identify(&[1])?;
    let sealed = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?
    .append(AdmittedSegmentRecord::for_chunk(&[1])?)?
    .append(AdmittedSegmentRecord::for_layout(&layout_one)?)?
    .append(AdmittedSegmentRecord::for_chunk(&[2])?)?
    .seal()?;
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
    publish_two_anchor_root(sandbox.path())?;
    Ok(sandbox)
}

pub(super) fn publish_two_anchor_root(root: &Path) -> Result<(), Box<dyn Error>> {
    let (blob_zero, layout_zero, _) = identify(&[0])?;
    let (blob_one, layout_one, _) = identify(&[1])?;
    let retention_root = RetentionRoot::new(
        RetentionNamespace::try_from(vec![0x2f])?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(1024, 8, 1 << 20, 1 << 24)?,
        ),
        None,
        vec![
            RetentionAnchor::new(blob_zero, layout_zero),
            RetentionAnchor::new(blob_one, layout_one),
        ],
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

pub(super) fn observe(root: &Path) -> Result<CompactionObservation, Box<dyn Error>> {
    let view =
        FilesystemRetentionSnapshot::load(root, catalog_policy()?, ReaderAttemptLimit::DEFAULT)?;
    observe_compaction(root, &view, catalog_policy()?).map_err(Into::into)
}

pub(super) fn plan(root: &Path) -> Result<CompactionPlan, Box<dyn Error>> {
    plan_compaction(&observe(root)?).map_err(Into::into)
}

pub(super) fn authority(root: &Path) -> Result<FilesystemCompactionAuthority, Box<dyn Error>> {
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(root)?;
    FilesystemCompactionAuthority::open(admission, root, catalog_policy()?).map_err(Into::into)
}

pub(super) fn gc_plan(root: &Path) -> Result<GcPlan, Box<dyn Error>> {
    let view =
        FilesystemRetentionSnapshot::load(root, catalog_policy()?, ReaderAttemptLimit::DEFAULT)?;
    let liveness = observe_gc_liveness(root, &view, catalog_policy()?)?;
    Ok(plan_gc(&liveness, GcLimits::MAXIMUM)?)
}

/// Every retained closure's root digest and member identities, and every
/// live record's exact bytes: the logical view compaction must leave
/// untouched. The closure *digest* is a transcript naming the physical
/// catalog coordinate, so it changes with every catalog successor by design.
pub(super) type Closures = BTreeMap<[u8; 32], ([u8; 32], Vec<SegmentRecordIdentity>)>;

pub(super) fn logical_view(root: &Path) -> Result<LogicalView, Box<dyn Error>> {
    let view =
        FilesystemRetentionSnapshot::load(root, catalog_policy()?, ReaderAttemptLimit::DEFAULT)?;
    let catalog_view = view.catalog().snapshot()?;
    let mut closures = Closures::new();
    crate::adapters::gc::visit_retained_closures(
        &view,
        &catalog_view,
        |namespace, root, members| {
            closures.insert(
                *namespace.as_bytes(),
                (
                    *root.digest().as_bytes(),
                    members.identities.iter().copied().collect(),
                ),
            );
            Ok(())
        },
    )?;
    drop(catalog_view);
    let observation = observe_compaction(root, &view, catalog_policy()?)?;
    let catalog = FilesystemCatalogSnapshot::load(root, catalog_policy()?)?;
    let snapshot = catalog.snapshot()?;
    let mut records = BTreeMap::new();
    for identity in observation.live() {
        let record = snapshot.record(*identity).ok_or("live record unnamed")?;
        let mut bytes = record.header().encode().to_vec();
        bytes.extend_from_slice(record.payload());
        bytes.extend_from_slice(record.checksum().as_bytes());
        records.insert(*identity, bytes);
    }
    Ok((closures, records))
}

pub(super) fn head_generation(root: &Path) -> Result<u64, Box<dyn Error>> {
    Ok(FilesystemCatalogSnapshot::load(root, catalog_policy()?)?
        .generation()
        .get())
}

/// Runs recovery over `root` and reports whether it found no residue.
pub(super) fn recovery_is_idle(root: &Path) -> Result<bool, Box<dyn Error>> {
    Ok(super::recover_compaction_unchecked_for_tests(root, catalog_policy()?)?.was_idle())
}
