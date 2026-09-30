//! A compactor death injected before each publication phase leaves exactly
//! the documented residue, which recovers to one lawful state, after which
//! the same successor is reached.

use std::error::Error;
use std::io;

use super::test_fixture::{
    authority, gc_plan, head_generation, logical_view, mixed_store, observe, plan,
};
use super::{
    CompactionRefusal, FilesystemCompactionError, plan_compaction,
    recover_compaction_unchecked_for_tests,
};
use crate::adapters::retention::filesystem_retention_test_fixture::catalog_policy;
use crate::adapters::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationPhase, CatalogPublicationReadiness, CatalogPublicationStorage,
    CatalogSnapshot, ChecksummedCatalog, FilesystemCatalogPublisher, RecoveryStage,
    SegmentPublication, publish_catalog_generation,
};

/// The complete publication order with one staged segment.
const PHASES: [CatalogPublicationPhase; 22] = [
    CatalogPublicationPhase::VerifyCurrent,
    CatalogPublicationPhase::LinkSegment,
    CatalogPublicationPhase::VerifySegmentPool,
    CatalogPublicationPhase::SynchronizeSegments,
    CatalogPublicationPhase::RemoveSegmentStage,
    CatalogPublicationPhase::SynchronizeStagingAfterSegment,
    CatalogPublicationPhase::CreateCatalogStage,
    CatalogPublicationPhase::WriteCatalog,
    CatalogPublicationPhase::FlushCatalog,
    CatalogPublicationPhase::SynchronizeCatalog,
    CatalogPublicationPhase::LinkCatalog,
    CatalogPublicationPhase::VerifyCatalogPool,
    CatalogPublicationPhase::SynchronizeCatalogs,
    CatalogPublicationPhase::RemoveCatalogStage,
    CatalogPublicationPhase::SynchronizeStagingAfterCatalog,
    CatalogPublicationPhase::CreateHeadStage,
    CatalogPublicationPhase::WriteHead,
    CatalogPublicationPhase::FlushHead,
    CatalogPublicationPhase::SynchronizeHead,
    CatalogPublicationPhase::VerifyHeadView,
    CatalogPublicationPhase::ReplaceHead,
    CatalogPublicationPhase::SynchronizeRoot,
];

/// Delegates to the publisher and refuses exactly one phase.
struct FailingAt<'publisher> {
    inner: &'publisher mut FilesystemCatalogPublisher,
    failing: CatalogPublicationPhase,
}

impl FailingAt<'_> {
    fn gate(&self, phase: CatalogPublicationPhase) -> io::Result<()> {
        if phase == self.failing {
            Err(io::Error::other("injected process death"))
        } else {
            Ok(())
        }
    }
}

macro_rules! forward {
    ($($name:ident => $phase:ident),* $(,)?) => {
        $(fn $name(&mut self) -> io::Result<()> {
            self.gate(CatalogPublicationPhase::$phase)?;
            self.inner.$name()
        })*
    };
}

impl CatalogPublicationStorage for FailingAt<'_> {
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
        segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        self.gate(CatalogPublicationPhase::VerifyCurrent)?;
        self.inner.verify_current(expected, candidate, segment)
    }

    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::LinkSegment)?;
        self.inner.link_segment(segment)
    }

    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::VerifySegmentPool)?;
        self.inner.verify_segment_pool(segment)
    }

    forward!(
        synchronize_segments => SynchronizeSegments,
        remove_segment_stage => RemoveSegmentStage,
        synchronize_staging_after_segment => SynchronizeStagingAfterSegment,
        create_catalog_stage => CreateCatalogStage,
        flush_catalog => FlushCatalog,
        synchronize_catalog => SynchronizeCatalog,
        synchronize_catalogs => SynchronizeCatalogs,
        remove_catalog_stage => RemoveCatalogStage,
        synchronize_staging_after_catalog => SynchronizeStagingAfterCatalog,
        create_head_stage => CreateHeadStage,
        flush_head => FlushHead,
        synchronize_head => SynchronizeHead,
        replace_head => ReplaceHead,
        synchronize_root => SynchronizeRoot,
    );

    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::WriteCatalog)?;
        self.inner.write_catalog(catalog)
    }

    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::LinkCatalog)?;
        self.inner.link_catalog(catalog)
    }

    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::VerifyCatalogPool)?;
        self.inner.verify_catalog_pool(catalog)
    }

    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::WriteHead)?;
        self.inner.write_head(head)
    }

    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        self.gate(CatalogPublicationPhase::VerifyHeadView)?;
        self.inner.verify_head_view(head, snapshot)
    }
}

/// The residue a death before phase `index` leaves, and whether `HEAD` has
/// already advanced.
fn expected_recovery(index: usize) -> (Vec<RecoveryStage>, bool, bool) {
    match index {
        0..=4 => (vec![RecoveryStage::Segment], false, false),
        7..=13 => (vec![RecoveryStage::Catalog], false, false),
        16 => (vec![RecoveryStage::NextHead], false, false),
        17..=20 => (vec![], true, true),
        21 => (vec![], false, true),
        _ => (vec![], false, false),
    }
}

#[test]
fn every_interrupted_publication_phase_recovers_to_one_lawful_state() -> Result<(), Box<dyn Error>>
{
    for (index, phase) in PHASES.into_iter().enumerate() {
        let sandbox = mixed_store(&format!("compaction-interrupt-{index}"))?;
        let planned = plan(sandbox.path())?;
        let before = logical_view(sandbox.path())?;
        let mut compactor = authority(sandbox.path())?;
        let error = compactor
            .execute_with(
                &planned,
                &mut |publisher, expectation, segment, catalog, segments| {
                    let mut failing = FailingAt {
                        inner: publisher,
                        failing: phase,
                    };
                    publish_catalog_generation(
                        &mut failing,
                        expectation,
                        segment,
                        catalog,
                        segments,
                    )
                },
            )
            .err()
            .ok_or_else(|| format!("{phase:?}: injected death did not refuse"))?;
        assert!(
            matches!(error, FilesystemCompactionError::Publish(_)),
            "{phase:?}: {error}"
        );
        drop(compactor);

        // Readers still admit the store and every closure still verifies.
        assert_eq!(logical_view(sandbox.path())?, before, "{phase:?}");

        let (discarded, finalized, advanced) = expected_recovery(index);
        let recovery = recover_compaction_unchecked_for_tests(sandbox.path(), catalog_policy()?)?;
        assert_eq!(recovery.discarded(), discarded.as_slice(), "{phase:?}");
        assert_eq!(recovery.finalized().is_some(), finalized, "{phase:?}");
        assert!(!sandbox.path().join("head.next").exists(), "{phase:?}");
        assert!(
            !sandbox.path().join("staging").join("current.seg").exists(),
            "{phase:?}"
        );
        assert!(
            !sandbox.path().join("staging").join("current.cat").exists(),
            "{phase:?}"
        );
        assert_eq!(
            head_generation(sandbox.path())?,
            if advanced { 3 } else { 2 },
            "{phase:?}"
        );
        assert_eq!(logical_view(sandbox.path())?, before, "{phase:?}");
        assert!(
            recover_compaction_unchecked_for_tests(sandbox.path(), catalog_policy()?)?.was_idle()
        );

        if advanced {
            assert!(matches!(
                plan_compaction(&observe(sandbox.path())?),
                Err(CompactionRefusal::NothingToCompact)
            ));
        } else {
            let again = plan(sandbox.path())?;
            assert_eq!(
                again, planned,
                "{phase:?}: the plan survives the interruption"
            );
            let _receipt = authority(sandbox.path())?.execute(&again)?;
            assert_eq!(head_generation(sandbox.path())?, 3, "{phase:?}");
        }
        assert_eq!(logical_view(sandbox.path())?, before, "{phase:?}");
        assert_eq!(gc_plan(sandbox.path())?.candidate_count(), 1, "{phase:?}");
        sandbox.remove()?;
    }
    Ok(())
}
