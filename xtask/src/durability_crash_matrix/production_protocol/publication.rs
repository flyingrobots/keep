//! This module owns execution of the production publication protocols.
//!
//! The child initializes a store directly and publishes its first catalog
//! through `FilesystemCatalogPublisher` and the retained-closure
//! gate. Segment staging boundaries are observed through the segment-stage
//! port and every pool, catalog, and head boundary through the writer's own
//! catalog observer, so no transition is reimplemented here.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CanonicalPublicationHead,
    CatalogGeneration, CatalogPublicationBoundary, CatalogPublicationExpectation,
    CatalogPublicationObserver, CatalogPublicationPhase, FilesystemCatalogPublisher,
    FilesystemSegmentStage, ObservedSegmentStage, SealedSegment, SegmentRecordLimit, StagedSegment,
    publish_retained_catalog_generation,
};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};
use super::segment_stage::CrashSegmentObserver;
use super::writer;
use super::{DurabilityCrashMatrixError, verification};

const CATALOG_INTERRUPTION: usize = 176;
const HEAD_INTERRUPTION: usize = 64;
const CATALOG_STAGE: &str = "staging/current.cat";
const HEAD_STAGE: &str = "head.next";

pub(super) fn run(
    store_root: &Path,
    mut control: CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    let mut publisher = writer::publisher(store_root)?;
    let sealed = stage_observed(&publisher, &mut control)?;
    let segment_bytes = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read sealed crash segment", source))?;
    let segments = [
        AdmittedSegment::decode(&segment_bytes, writer::segment_policy())
            .map_err(|source| verification("admit sealed crash segment", source))?,
    ];
    let catalog = first_catalog(&segments)?;
    let selection = publisher
        .select_segment(sealed, &segments[0])
        .map_err(|source| verification("select production segment stage", source))?;
    let torn = TornWrites::new(store_root, &catalog)?;
    publisher.observe_catalog_phases_for_repository_tasks(observer(control, torn));
    publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )
    .map(|_receipt| ())
    .map_err(|source| verification("execute production catalog publication", source))
}

/// Encodes generation one over `segments`, the catalog every crash row publishes.
pub(super) fn first_catalog(
    segments: &[AdmittedSegment<'_>],
) -> Result<CanonicalCatalog, DurabilityCrashMatrixError> {
    let generation = CatalogGeneration::new(1)
        .map_err(|source| verification("construct crash catalog generation", source))?;
    CanonicalCatalog::from_segments(generation, None, segments)
        .map_err(|source| verification("encode crash catalog", source))
}

/// Stages and seals the one-zero segment with no crash observation.
pub(super) fn stage_one_zero(
    publisher: &FilesystemCatalogPublisher,
) -> Result<SealedSegment<FilesystemSegmentStage<'_>>, DurabilityCrashMatrixError> {
    let stage = publisher
        .create_segment_stage()
        .map_err(|source| verification("create production segment stage", source))?;
    StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
        .map_err(|source| verification("write production segment header", source))?
        .append(one_zero_record()?)
        .map_err(|source| verification("append production segment record", source))?
        .seal()
        .map_err(|source| verification("seal production segment", source))
}

fn stage_observed<'publisher>(
    publisher: &'publisher FilesystemCatalogPublisher,
    control: &mut CrashControl,
) -> Result<SealedSegment<FilesystemSegmentStage<'publisher>>, DurabilityCrashMatrixError> {
    let point = DurabilityCrashPoint::CreateSegmentStage;
    control
        .before(point, DuringTiming::After)
        .map_err(crash_gate)?;
    let stage = publisher
        .create_segment_stage()
        .map_err(|source| verification("create production segment stage", source))?;
    control
        .after(point, DuringTiming::After)
        .map_err(crash_gate)?;
    let stage = ObservedSegmentStage::new(stage, CrashSegmentObserver::new(control));
    Ok(StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
        .map_err(|source| verification("write production segment header", source))?
        .append(one_zero_record()?)
        .map_err(|source| verification("append production segment record", source))?
        .seal()
        .map_err(|source| verification("seal production segment", source))?
        .without_observer())
}

fn one_zero_record() -> Result<AdmittedSegmentRecord<'static>, DurabilityCrashMatrixError> {
    AdmittedSegmentRecord::for_chunk(&[0])
        .map_err(|source| verification("admit crash segment record", source))
}

/// The strict prefixes a `during` write leaves in each fixed stage.
struct TornWrites {
    catalog_stage: PathBuf,
    catalog: Vec<u8>,
    head_stage: PathBuf,
    head: Vec<u8>,
}

impl TornWrites {
    fn new(
        store_root: &Path,
        catalog: &CanonicalCatalog,
    ) -> Result<Self, DurabilityCrashMatrixError> {
        let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
        Ok(Self {
            catalog_stage: store_root.join(CATALOG_STAGE),
            catalog: prefix(catalog.encoded(), CATALOG_INTERRUPTION)?,
            head_stage: store_root.join(HEAD_STAGE),
            head: prefix(head.encoded(), HEAD_INTERRUPTION)?,
        })
    }

    fn select(
        &self,
        phase: CatalogPublicationPhase,
    ) -> Option<(DurabilityCrashPoint, &Path, &[u8])> {
        match phase {
            CatalogPublicationPhase::WriteCatalog => Some((
                DurabilityCrashPoint::WriteCatalog,
                &self.catalog_stage,
                &self.catalog,
            )),
            CatalogPublicationPhase::WriteHead => Some((
                DurabilityCrashPoint::WriteHead,
                &self.head_stage,
                &self.head,
            )),
            _ => None,
        }
    }
}

fn prefix(encoded: &[u8], length: usize) -> Result<Vec<u8>, DurabilityCrashMatrixError> {
    encoded
        .get(..length)
        .map(<[u8]>::to_vec)
        .ok_or(DurabilityCrashMatrixError::FixtureRange)
}

/// Maps the writer's own transitions onto `KEEP-CRASH-009` through `-026`.
fn observer(mut control: CrashControl, torn: TornWrites) -> CatalogPublicationObserver {
    Box::new(move |phase, boundary| {
        if let Some((point, stage, bytes)) = torn.select(phase) {
            return interrupt_write(&mut control, point, boundary, stage, bytes);
        }
        let Some((point, timing)) = crash_point(phase) else {
            return Ok(());
        };
        match boundary {
            CatalogPublicationBoundary::Before => control.before(point, timing),
            CatalogPublicationBoundary::After => control.after(point, timing),
        }
    })
}

/// Dies before, inside, or after one stage write.
///
/// The `during` case is the crash itself: the strict prefix reaches the stage
/// file the writer created and the process dies before the writer's own
/// write runs.
fn interrupt_write(
    control: &mut CrashControl,
    point: DurabilityCrashPoint,
    boundary: CatalogPublicationBoundary,
    stage: &Path,
    bytes: &[u8],
) -> io::Result<()> {
    match (boundary, control.position(point)) {
        (CatalogPublicationBoundary::Before, Some(DurabilityCrashPosition::Before))
        | (CatalogPublicationBoundary::After, Some(DurabilityCrashPosition::After)) => {
            control.await_process_death()
        }
        (CatalogPublicationBoundary::Before, Some(DurabilityCrashPosition::During)) => {
            fs::OpenOptions::new()
                .write(true)
                .open(stage)?
                .write_all(bytes)?;
            control.await_process_death()
        }
        _ => Ok(()),
    }
}

const fn crash_point(
    phase: CatalogPublicationPhase,
) -> Option<(DurabilityCrashPoint, DuringTiming)> {
    use CatalogPublicationPhase as Phase;
    use DurabilityCrashPoint as Point;
    let mapped = match phase {
        Phase::LinkSegment => (Point::LinkSegment, DuringTiming::After),
        Phase::SynchronizeSegments => (Point::SynchronizeSegmentPool, DuringTiming::Before),
        Phase::RemoveSegmentStage => (Point::RemoveSegmentStage, DuringTiming::After),
        Phase::SynchronizeStagingAfterSegment => {
            (Point::SynchronizeStagingAfterSegment, DuringTiming::Before)
        }
        Phase::CreateCatalogStage => (Point::CreateCatalogStage, DuringTiming::After),
        Phase::FlushCatalog => (Point::FlushCatalog, DuringTiming::Before),
        Phase::SynchronizeCatalog => (Point::SynchronizeCatalog, DuringTiming::Before),
        Phase::LinkCatalog => (Point::LinkCatalog, DuringTiming::After),
        Phase::SynchronizeCatalogs => (Point::SynchronizeCatalogPool, DuringTiming::Before),
        Phase::RemoveCatalogStage => (Point::RemoveCatalogStage, DuringTiming::After),
        Phase::SynchronizeStagingAfterCatalog => {
            (Point::SynchronizeStagingAfterCatalog, DuringTiming::Before)
        }
        Phase::CreateHeadStage => (Point::CreateHeadStage, DuringTiming::After),
        Phase::FlushHead => (Point::FlushHead, DuringTiming::Before),
        Phase::SynchronizeHead => (Point::SynchronizeHead, DuringTiming::Before),
        Phase::ReplaceHead => (Point::ReplaceHead, DuringTiming::After),
        Phase::SynchronizeRoot => (Point::SynchronizeRootAfterHead, DuringTiming::Before),
        _ => return None,
    };
    Some(mapped)
}

const fn crash_gate(source: io::Error) -> DurabilityCrashMatrixError {
    DurabilityCrashMatrixError::io("await production crash boundary", source)
}
