//! This module owns filesystem compaction under writer authority: re-prove
//! the plan, copy the live records into one new sealed segment, publish the
//! successor through the complete catalog protocol, and revalidate.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;

use super::plan::superseded_set;
use super::{CompactionPlan, FilesystemCompactionError as Error, plan_compaction};
use crate::adapters::gc::{GcLimits, GcSegmentClassification, GcUnreachableEvidence, plan_gc};
use crate::adapters::{
    AdmittedSegment, CanonicalCatalog, CatalogPublicationError, CatalogPublicationExpectation,
    CatalogPublicationReceipt, CatalogRestartPolicy, FilesystemCatalogPublisher,
    FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission, ReaderAttemptLimit, SegmentDigest,
    SegmentPublication, SegmentRecordLimit, StagedSegment, physical_pool_name,
    publish_catalog_generation,
};
use crate::{CatalogDigest, CatalogGeneration};

/// The observed artifact about to be reread under compaction authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReadTarget {
    Retained(SegmentDigest),
    Staged,
}

/// The publication call compaction drives: production passes
/// [`publish_catalog_generation`] on the authority's own publisher; a test
/// or crash harness wraps that publisher in a fault-injecting storage.
pub type CompactionPublish<'call> = &'call mut dyn for<'p, 's, 'r, 'c> FnMut(
    &'p mut FilesystemCatalogPublisher,
    CatalogPublicationExpectation,
    SegmentPublication<'s, 'r>,
    &CanonicalCatalog,
    &[AdmittedSegment<'c>],
) -> Result<
    CatalogPublicationReceipt,
    CatalogPublicationError,
>;

/// What one completed compaction established.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionReceipt {
    plan: CompactionPlan,
    generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    new_segment: Option<SegmentDigest>,
}

impl CompactionReceipt {
    /// The plan that was executed.
    pub const fn plan(&self) -> &CompactionPlan {
        &self.plan
    }

    /// The published successor generation.
    pub const fn generation(&self) -> CatalogGeneration {
        self.generation
    }

    /// The published successor digest.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.catalog_digest
    }

    /// The new segment holding the copied records, when any were copied.
    #[must_use]
    pub const fn new_segment(&self) -> Option<SegmentDigest> {
        self.new_segment
    }

    /// The segments the successor no longer names: GC's next candidates.
    #[must_use]
    pub fn superseded(&self) -> BTreeSet<SegmentDigest> {
        superseded_set(&self.plan)
    }
}

/// Exclusive authority to compact one pinned version-two root.
///
/// The authority holds the writer lock through its catalog publisher for
/// its lifetime. Publication proceeds beside readers because it only adds
/// an immutable segment and a catalog successor; GC later takes the
/// exclusive reader fence to retire the superseded segments.
#[must_use]
pub struct FilesystemCompactionAuthority {
    publisher: FilesystemCatalogPublisher,
    policy: CatalogRestartPolicy,
}

impl FilesystemCompactionAuthority {
    /// Pins one admitted version-two root for compaction.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemCompactionError::Observe`](super::FilesystemCompactionError::Observe) when the publication
    /// directories cannot be pinned.
    pub fn open(
        admission: FilesystemVersionTwoAdmission,
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, Error> {
        let publisher = FilesystemCatalogPublisher::open_version_two(admission, policy)
            .map_err(|source| Error::Observe { source })?;
        crate::adapters::filesystem_root_binding::require_locator(&publisher.root, store_root)
            .map_err(|source| Error::Observe { source })?;
        Ok(Self { publisher, policy })
    }

    /// Executes `plan` completely.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemCompactionError`](super::FilesystemCompactionError) at the exact refusal; a
    /// publication refusal leaves the completed phases' residue for
    /// [`recover_compaction`](super::recover_compaction).
    pub fn execute(&mut self, plan: &CompactionPlan) -> Result<CompactionReceipt, Error> {
        self.execute_with(plan, &mut |publisher,
                                      expectation,
                                      segment,
                                      catalog,
                                      segments| {
            publish_catalog_generation(publisher, expectation, segment, catalog, segments)
        })
    }

    /// Executes `plan` with `publish` driving the catalog protocol.
    ///
    /// The plan is re-proven against the reopened store first; the live
    /// records are copied into `staging/current.seg` in canonical identity
    /// order and sealed; the successor names every retained segment and the
    /// new one; `publish` runs the protocol; then the store is reopened and
    /// every superseded segment must plan as a superseded GC candidate.
    ///
    /// Retained segments are reread at the exact lengths admitted by the
    /// bounded catalog load; the new stage is reread at its sealed receipt's
    /// exact length. Metadata growth refuses before content allocation, and
    /// growth after that check cannot extend the bounded read. The operation
    /// materializes these bytes while retaining the loaded catalog snapshot.
    ///
    /// # Errors
    ///
    /// As [`Self::execute`]. [`FilesystemCompactionError::Materialize`](super::FilesystemCompactionError::Materialize)
    /// preserves typed length, allocation and I/O failures from a reread.
    #[doc(hidden)]
    pub fn execute_with(
        &mut self,
        plan: &CompactionPlan,
        publish: CompactionPublish<'_>,
    ) -> Result<CompactionReceipt, Error> {
        self.execute_before_read(plan, publish, &mut |_target| Ok(()))
    }

    pub(super) fn execute_before_read(
        &mut self,
        plan: &CompactionPlan,
        publish: CompactionPublish<'_>,
        before_read: &mut dyn FnMut(ReadTarget) -> io::Result<()>,
    ) -> Result<CompactionReceipt, Error> {
        self.reprove(plan)?;
        let catalog = crate::adapters::catalog_restart_loader::load_from_directory(
            &self.publisher.root,
            "HEAD",
            self.policy,
        )
        .map_err(|source| Error::Catalog(Box::new(source)))?;
        let snapshot = catalog
            .snapshot()
            .map_err(|source| Error::Catalog(Box::new(source)))?;
        let retained_bytes = self.read_retained(plan, &catalog, before_read)?;
        let mut segments = Vec::new();
        for bytes in &retained_bytes {
            segments.push(admit(bytes, self.policy)?);
        }
        let staged = if plan.copied_records() == 0 {
            None
        } else {
            Some(self.stage_copies(plan, &snapshot, before_read)?)
        };
        let (new_bytes, sealed) = match staged {
            Some((bytes, sealed)) => (Some(bytes), Some(sealed)),
            None => (None, None),
        };
        let new_segment = new_bytes
            .as_deref()
            .map(|bytes| admit(bytes, self.policy))
            .transpose()?;
        let selection = match (sealed, &new_segment) {
            (Some(sealed), Some(admitted)) => self
                .publisher
                .select_segment(sealed, admitted)
                .map_err(Error::Selection)?,
            _ => SegmentPublication::none(),
        };
        if let Some(admitted) = new_segment.as_ref() {
            segments.push(
                AdmittedSegment::decode(admitted.encoded(), self.policy.segment_read())
                    .map_err(|source| Error::Segment(Box::new(source)))?,
            );
        }
        let successor = CanonicalCatalog::from_segments(
            plan.successor_generation(),
            Some(plan.coordinates().catalog_digest()),
            &segments,
        )
        .map_err(|source| Error::Successor(Box::new(source)))?;
        let expectation = CatalogPublicationExpectation::successor_of(&snapshot);
        let receipt = publish(
            &mut self.publisher,
            expectation,
            selection,
            &successor,
            &segments,
        )
        .map_err(|source| Error::Publish(Box::new(source)))?;
        drop(segments);
        drop(snapshot);
        self.revalidate(plan)?;
        Ok(CompactionReceipt {
            plan: plan.clone(),
            generation: receipt.generation(),
            catalog_digest: receipt.catalog_digest(),
            new_segment: new_segment.as_ref().map(AdmittedSegment::digest),
        })
    }

    fn view(&self) -> Result<FilesystemRetentionSnapshot, Error> {
        FilesystemRetentionSnapshot::load_under_writer_authority(
            &self.publisher.root,
            self.policy,
            ReaderAttemptLimit::DEFAULT,
        )
        .map_err(|source| Error::Snapshot(Box::new(source)))
    }

    fn reprove(&self, plan: &CompactionPlan) -> Result<(), Error> {
        let view = self.view()?;
        let observation = super::observation::observe_compaction_from_view(&view, self.policy)
            .map_err(|source| Error::Liveness(Box::new(source)))?;
        let fresh = plan_compaction(&observation).map_err(Error::Refused)?;
        if fresh == *plan {
            Ok(())
        } else {
            Err(Error::PlanStale)
        }
    }

    fn read_retained(
        &self,
        plan: &CompactionPlan,
        catalog: &crate::adapters::FilesystemCatalogSnapshot,
        before_read: &mut dyn FnMut(ReadTarget) -> io::Result<()>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        let mut retained = Vec::new();
        for digest in plan.retained() {
            before_read(ReadTarget::Retained(digest))
                .map_err(|source| Error::Observe { source })?;
            let name = physical_pool_name::segment(digest);
            let loaded = catalog
                .loaded_segments()
                .iter()
                .find(|loaded| loaded.digest() == digest)
                .ok_or(Error::PlanStale)?;
            let expected = admit(loaded.encoded(), self.policy)?.segment_length();
            let bytes =
                super::materialization::read(&self.publisher.segments, &name, digest, expected)?;
            retained.push(bytes);
        }
        Ok(retained)
    }

    /// Copies every planned live record into a fresh sealed stage and
    /// returns the stage's exact bytes with the sealed handle.
    fn stage_copies<'publisher>(
        &'publisher self,
        plan: &CompactionPlan,
        snapshot: &crate::adapters::CatalogSnapshot<'_, '_, '_>,
        before_read: &mut dyn FnMut(ReadTarget) -> io::Result<()>,
    ) -> Result<
        (
            Vec<u8>,
            crate::adapters::SealedSegment<crate::adapters::FilesystemSegmentStage<'publisher>>,
        ),
        Error,
    > {
        let stage = self
            .publisher
            .create_segment_stage()
            .map_err(|source| match source {
                crate::adapters::SegmentStageCreateError::Create { source } => {
                    Error::Observe { source }
                }
            })?;
        let mut staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
            .map_err(|source| Error::Stage(Box::new(source)))?;
        let copies: BTreeSet<_> = plan.copied().collect();
        for identity in copies {
            let record = snapshot.record(identity).ok_or(Error::RecordVanished)?;
            staged = staged
                .append(record)
                .map_err(|source| Error::Stage(Box::new(source)))?;
        }
        let sealed = staged
            .seal()
            .map_err(|source| Error::Stage(Box::new(source)))?;
        before_read(ReadTarget::Staged).map_err(|source| Error::Observe { source })?;
        let bytes = super::materialization::read(
            &self.publisher.staging,
            crate::adapters::filesystem_catalog_publisher::CURRENT_SEGMENT,
            sealed.digest(),
            sealed.segment_length(),
        )?;
        Ok((bytes, sealed))
    }

    /// After publication every superseded segment must be a superseded
    /// retirement candidate and every retained closure must still verify.
    fn revalidate(&self, plan: &CompactionPlan) -> Result<(), Error> {
        let view = self.view()?;
        let liveness = crate::adapters::gc::observe_gc_liveness_from_view(&view, self.policy)
            .map_err(|source| Error::Liveness(Box::new(source)))?;
        let gc_plan = plan_gc(&liveness, GcLimits::MAXIMUM).map_err(Error::Revalidation)?;
        let superseded: BTreeMap<SegmentDigest, GcSegmentClassification> = superseded_set(plan)
            .into_iter()
            .filter_map(|digest| gc_plan.classification(digest).map(|c| (digest, c)))
            .collect();
        let expected = GcSegmentClassification::Unreachable(GcUnreachableEvidence::Superseded);
        if superseded.len() == superseded_set(plan).len()
            && superseded
                .values()
                .all(|classification| *classification == expected)
        {
            Ok(())
        } else {
            Err(Error::NotSuperseded)
        }
    }
}

fn admit(bytes: &[u8], policy: CatalogRestartPolicy) -> Result<AdmittedSegment<'_>, Error> {
    AdmittedSegment::decode(bytes, policy.segment_read())
        .map_err(|source| Error::Segment(Box::new(source)))
}
