//! This module owns populated-store construction for test code.
//!
//! Every populated version-two test store is created by [`create`] and then
//! extended by [`retain`] and by the version-two writer in `successor_catalog.rs`.
//! Only public production entry points run, so test code is not a second
//! implementation of the store format. Today [`create`] initializes version
//! one, publishes the first catalog through `FilesystemCatalogPublisher`, and
//! migrates. When direct initialization replaces migration (#198), only that
//! creation step changes.
//!
//! Consumers include this file by path beside their laws, so it names the
//! crate as `keep` and uses `super`-relative visibility.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use keep::{
    AdmittedLayout, AdmittedRetentionManifest, AdmittedRetentionRoot, AdmittedSegment,
    AdmittedSegmentRecord, BlobId, CanonicalCatalog, CanonicalLayoutRecord, CanonicalRetentionRoot,
    CanonicalStoreMigrationReceipt, CatalogGeneration, CatalogPublicationExpectation,
    CatalogPublicationReceipt, CatalogRestartByteLimit, CatalogRestartPolicy, ChunkId, ChunkSpan,
    FastCdc, FilesystemCatalogPublisher, FilesystemCatalogSnapshot, FilesystemPlatformAdmission,
    FilesystemRetentionPublicationAuthority, FilesystemStoreMigrationAuthority,
    FilesystemVersionTwoAdmission, LayoutEntryLimit, RegisteredRetentionProfile,
    RegisteredStorageProfile, RetentionAnchor, RetentionClosureLimits,
    RetentionGenerationExpectation, RetentionNamespace, RetentionPolicy,
    RetentionPublicationReceipt, RetentionRoot, RootGeneration, SegmentReadPolicy,
    SegmentRecordLimit, StagedSegment, execute_retention_publication, execute_store_migration,
    preflight_retention_transition, prepare_retention_publication, publish_catalog_generation,
};

pub(super) type TestResult<T> = Result<T, Box<dyn Error>>;

/// How the builder acquires writer authority over a store root.
///
/// [`Self::PLATFORM`] is the production platform profile. Private unit tests
/// that run outside that profile supply their own unchecked route; every step
/// after admission is identical.
#[derive(Clone, Copy)]
pub(super) struct StoreAdmission {
    /// Initializes an empty root.
    pub(super) initialize: fn(&Path) -> TestResult<FilesystemPlatformAdmission>,
    /// Reacquires authority over the published pre-migration root.
    pub(super) reopen_version_one: fn(&Path) -> TestResult<FilesystemPlatformAdmission>,
    /// Reacquires authority over the created version-two root.
    pub(super) reopen_version_two: fn(&Path) -> TestResult<FilesystemVersionTwoAdmission>,
}

impl StoreAdmission {
    /// The production platform profile.
    pub(super) const PLATFORM: Self = Self {
        initialize: |root| Ok(FilesystemPlatformAdmission::initialize(root)?),
        reopen_version_one: |root| Ok(FilesystemPlatformAdmission::reopen(root)?),
        reopen_version_two: |root| Ok(FilesystemVersionTwoAdmission::reopen(root)?),
    };
}

/// Which records of the given contents the first catalog references.
///
/// Anything but [`Self::Complete`] deliberately builds a store whose catalog
/// is valid but whose content is incomplete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FirstCatalogRecords {
    /// Every distinct chunk, then each layout.
    Complete,
    /// Only the layout records.
    LayoutsOnly,
    /// One chunk record and every layout.
    OnlyChunk(ChunkId),
}

impl FirstCatalogRecords {
    fn keeps_chunk(self, chunk: ChunkId) -> bool {
        match self {
            Self::Complete => true,
            Self::LayoutsOnly => false,
            Self::OnlyChunk(selected) => selected == chunk,
        }
    }
}

/// The receipts of one completed creation step.
#[must_use]
pub(super) struct CreatedStore {
    /// The first catalog publication.
    pub(super) catalog: CatalogPublicationReceipt,
    /// The step that made the root version two.
    pub(super) migration: CanonicalStoreMigrationReceipt,
}

/// The canonical identity of one blob under the registered storage profile.
pub(super) struct Identified {
    pub(super) spans: Vec<ChunkSpan>,
    pub(super) record: CanonicalLayoutRecord,
    pub(super) target: BlobId,
}

/// Chunks `bytes` and encodes its canonical layout record.
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
        segment_policy(),
        CatalogRestartByteLimit::new(16_777_216)?,
    ))
}

pub(super) const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

/// Creates a version-two store at the empty directory `root`.
///
/// The first catalog holds one segment with the `selection` of `contents`'
/// records. Writer authority is released before this returns.
pub(super) fn create(
    root: &Path,
    admission: StoreAdmission,
    contents: &[&[u8]],
    selection: FirstCatalogRecords,
) -> TestResult<CreatedStore> {
    let identified = contents
        .iter()
        .map(|bytes| identify(bytes))
        .collect::<TestResult<Vec<_>>>()?;
    let blobs = contents.iter().copied().zip(&identified);
    let records = select_records(blobs, selection)?;
    let catalog = publish_first_catalog(root, (admission.initialize)(root)?, records)?;
    let migration = migrate(root, admission)?;
    Ok(CreatedStore { catalog, migration })
}

/// Makes the published root version two and releases writer authority.
fn migrate(root: &Path, admission: StoreAdmission) -> TestResult<CanonicalStoreMigrationReceipt> {
    let mut authority = FilesystemStoreMigrationAuthority::open(
        (admission.reopen_version_one)(root)?,
        segment_policy(),
    )?;
    let intent = authority.observe_intent()?;
    let receipt = execute_store_migration(&mut authority, &intent)?;
    drop(authority);
    Ok(receipt)
}

/// Every selected chunk record, deduplicated across blobs, then each new layout.
pub(super) fn select_records<'content>(
    blobs: impl IntoIterator<Item = (&'content [u8], &'content Identified)>,
    selection: FirstCatalogRecords,
) -> TestResult<Vec<AdmittedSegmentRecord<'content>>> {
    let mut chunks = BTreeSet::new();
    let mut layouts = BTreeSet::new();
    let mut records = Vec::new();
    for (bytes, identity) in blobs {
        for span in &identity.spans {
            if selection.keeps_chunk(span.id()) && chunks.insert(span.id()) {
                let start = usize::try_from(span.offset().get())?;
                let end = usize::try_from(span.end().get())?;
                let chunk = bytes.get(start..end).ok_or("chunk outside source")?;
                records.push(AdmittedSegmentRecord::for_chunk(chunk)?);
            }
        }
        if layouts.insert(identity.record.id()) {
            records.push(AdmittedSegmentRecord::for_layout(&identity.record)?);
        }
    }
    Ok(records)
}

fn publish_first_catalog(
    root: &Path,
    admission: FilesystemPlatformAdmission,
    records: Vec<AdmittedSegmentRecord<'_>>,
) -> TestResult<CatalogPublicationReceipt> {
    let mut publisher = FilesystemCatalogPublisher::open(admission, policy()?)?;
    let mut stage = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?;
    for record in records {
        stage = stage.append(record)?;
    }
    let sealed = stage.seal()?;
    let bytes = fs::read(root.join("staging/current.seg"))?;
    let segments = [AdmittedSegment::decode(&bytes, segment_policy())?];
    let segment = segments.first().ok_or("segment absent")?;
    let selected = publisher.select_segment(sealed, segment)?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    Ok(publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selected,
        &catalog,
        &segments,
    )?)
}

/// Publishes a generation-one root in `namespace` that anchors every blob.
pub(super) fn retain(
    root: &Path,
    admission: StoreAdmission,
    namespace: &[u8],
    contents: &[&[u8]],
) -> TestResult<RetentionPublicationReceipt> {
    let anchors = contents
        .iter()
        .map(|bytes| {
            let identity = identify(bytes)?;
            Ok(RetentionAnchor::new(identity.target, identity.record.id()))
        })
        .collect::<TestResult<BTreeSet<_>>>()?;
    let retained = RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 16_777_216, 67_108_864)?,
        ),
        None,
        anchors.into_iter().collect(),
    )?;
    let canonical = CanonicalRetentionRoot::from_root(&retained)?;
    let candidate = AdmittedRetentionRoot::decode(canonical.encoded())?;
    let current = FilesystemCatalogSnapshot::load(root, policy()?)?;
    let snapshot = current.snapshot()?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )?;
    let mut authority =
        FilesystemRetentionPublicationAuthority::open((admission.reopen_version_two)(root)?)?;
    let observed = authority.observe_current()?;
    let manifest = observed
        .as_ref()
        .map(|state| AdmittedRetentionManifest::decode(state.manifest_bytes()))
        .transpose()?;
    let prepared = prepare_retention_publication(preflight, manifest.as_ref())?;
    Ok(execute_retention_publication(&mut authority, &prepared)?)
}

/// Requires the first catalog's pools and head to hold exactly these bytes.
///
/// Laws that prepare against frozen corpus bytes call this, so a writer whose
/// output drifts refuses at construction instead of failing an unrelated law.
pub(super) fn require_first_catalog(
    root: &Path,
    segment: &[u8],
    catalog: &[u8],
    head: &[u8],
) -> TestResult<()> {
    require_sole_entry(root, "segments", segment)?;
    require_sole_entry(root, "catalogs", catalog)?;
    if fs::read(root.join("HEAD"))? == head {
        Ok(())
    } else {
        Err("writer-built HEAD differs from the expected bytes".into())
    }
}

fn require_sole_entry(root: &Path, pool: &str, expected: &[u8]) -> TestResult<()> {
    let entries = fs::read_dir(root.join(pool))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    match entries.as_slice() {
        [only] if fs::read(only)? == expected => Ok(()),
        _ => Err(format!("writer-built {pool} pool is not exactly the expected entry").into()),
    }
}
