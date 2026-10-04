//! Filesystem GC execution and recovery laws over the migrated fixture
//! store: one disposed orphan retires, every refusal happens before any
//! intent, and every interrupted prefix recovers to the same complete
//! state without losing the live segment.

#[path = "filesystem_gc_effect_tests.rs"]
mod effect_tests;

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use super::liveness_observation_tests::{
    ORPHAN_SEGMENT_HEX, ORPHAN_SEGMENT_NAME, disposition_path, observe, published_store,
    segment_receipt,
};
use super::{
    FilesystemGcAuthority, FilesystemGcError, GcExecutionPhase, GcExecutionPoint,
    GcExecutionStorage, GcFixedStage, GcLimits, GcPlan, GcRecoveryPlan, plan_gc,
    resume_gc_execution,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, reopen_authority,
};
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission, ReaderAttemptLimit,
    RetentionCurrentStateRefusal, RetentionPublicationError,
};
use crate::execute_retention_publication;

const LIVE_SEGMENT_NAME: &str =
    "221f6745cd8a5221c9a87c3707593608479282b54a4a74d0e753fd76f70e8db2.seg";
/// Execution points for one candidate: the 14 phases with one unlink and
/// one pool synchronization.
const POINT_COUNT: usize = 14;

/// A published store holding one orphan pool segment and the exact retire
/// receipt that releases it.
fn disposed_store(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = published_store(name)?;
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;
    let coordinates = observe(sandbox.path())?.coordinates();
    let exact = segment_receipt(coordinates, coordinates.catalog_digest())?;
    fs::write(disposition_path(sandbox.path()), &exact)?;
    Ok(sandbox)
}

fn plan(root: &Path) -> Result<GcPlan, Box<dyn Error>> {
    plan_gc(&observe(root)?, GcLimits::MAXIMUM).map_err(Into::into)
}

fn authority(root: &Path) -> Result<FilesystemGcAuthority, Box<dyn Error>> {
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(root)?;
    FilesystemGcAuthority::open(admission, root, catalog_policy()?).map_err(Into::into)
}

fn gc_entries(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names: Vec<String> = fs::read_dir(root.join("gc"))?
        .map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, io::Error>>()?;
    names.sort();
    Ok(names)
}

fn assert_complete(root: &Path) -> Result<(), Box<dyn Error>> {
    assert_eq!(gc_entries(root)?, ["receipt"]);
    assert!(!root.join("segments").join(ORPHAN_SEGMENT_NAME).exists());
    assert!(root.join("segments").join(LIVE_SEGMENT_NAME).exists());
    let plan = plan(root)?;
    assert_eq!(plan.candidate_count(), 0);
    assert_eq!(plan.already_retired().len(), 1);
    let report = authority(root)?.recover()?;
    assert_eq!(report.plan(), GcRecoveryPlan::Complete);
    assert!(report.receipt().is_some());
    Ok(())
}

/// Executes exactly `remaining` points, then refuses like a process death.
struct Prefix<'authority> {
    inner: &'authority mut FilesystemGcAuthority,
    remaining: usize,
}

impl Prefix<'_> {
    fn step(&mut self) -> io::Result<()> {
        if self.remaining == 0 {
            return Err(io::Error::other("prefix exhausted"));
        }
        self.remaining = self.remaining.saturating_sub(1);
        Ok(())
    }
}

macro_rules! forward {
    ($($name:ident),* $(,)?) => {
        $(fn $name(&mut self) -> io::Result<()> {
            self.step()?;
            self.inner.$name()
        })*
    };
}

impl GcExecutionStorage for Prefix<'_> {
    forward!(
        write_intent_stage,
        synchronize_intent_stage,
        link_intent,
        synchronize_gc_after_intent,
        remove_intent_stage,
        synchronize_gc_after_intent_cleanup,
        write_receipt_stage,
        synchronize_receipt_stage,
        replace_receipt,
        synchronize_gc_after_receipt,
        remove_intent,
        synchronize_gc_after_intent_removal,
    );

    fn unlink_candidate(&mut self, index: usize) -> io::Result<()> {
        self.step()?;
        self.inner.unlink_candidate(index)
    }

    fn synchronize_segment_pool(&mut self, index: usize) -> io::Result<()> {
        self.step()?;
        self.inner.synchronize_segment_pool(index)
    }
}

/// Runs the first `count` points of a fresh retirement and releases the
/// writer, like a process death there.
fn interrupt(root: &Path, count: usize) -> Result<(), Box<dyn Error>> {
    let plan = plan(root)?;
    let mut authority = authority(root)?;
    let prepared = authority.prepare(&plan)?;
    let mut prefix = Prefix {
        inner: &mut authority,
        remaining: count,
    };
    let result = resume_gc_execution(
        &mut prefix,
        prepared.candidate_count(),
        GcExecutionPoint::START,
    );
    if count < POINT_COUNT {
        assert_eq!(
            result.err().map(|error| error.executed().len()),
            Some(count)
        );
    } else {
        assert_eq!(result?.executed().len(), POINT_COUNT);
    }
    Ok(())
}

/// The documented recovery for a process death after `count` points.
fn expected_plan(count: usize) -> GcRecoveryPlan {
    let resume = |phase| GcRecoveryPlan::Resume {
        from: GcExecutionPoint::at(phase),
    };
    match count {
        0 => GcRecoveryPlan::Idle,
        1 | 2 => resume(GcExecutionPhase::SynchronizeIntentStage),
        3 | 4 => resume(GcExecutionPhase::SynchronizeGcAfterIntent),
        5 | 6 => resume(GcExecutionPhase::SynchronizeGcAfterIntentCleanup),
        7 | 8 => resume(GcExecutionPhase::WriteReceiptStage),
        9 | 10 => resume(GcExecutionPhase::SynchronizeReceiptStage),
        11 | 12 => resume(GcExecutionPhase::SynchronizeGcAfterReceipt),
        _ => GcRecoveryPlan::Complete,
    }
}

#[test]
fn retiring_the_disposed_orphan_leaves_the_receipt_and_the_live_segment()
-> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-execute-retire")?;
    let plan = plan(sandbox.path())?;
    assert_eq!(plan.candidate_count(), 1);

    let receipt = authority(sandbox.path())?.execute(&plan)?;

    assert_eq!(receipt.receipt().generation().get(), 1);
    assert_eq!(receipt.receipt().synchronization_count(), 1);
    assert_complete(sandbox.path())?;
    let again = authority(sandbox.path())?.recover()?;
    assert_eq!(again.receipt(), Some(receipt.receipt()));
    // Nothing is left to retire, and the prior receipt stays in place.
    let error = authority(sandbox.path())?
        .execute(&plan)
        .err()
        .ok_or("a stale plan executed twice")?;
    assert!(matches!(error, FilesystemGcError::PlanStale), "{error}");
    assert_eq!(gc_entries(sandbox.path())?, ["receipt"]);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_second_retirement_succeeds_the_first_receipts_generation() -> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-execute-second")?;
    let first = authority(sandbox.path())?.execute(&plan(sandbox.path())?)?;
    // A second orphan disposed after the first retirement is generation two.
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;
    let plan = plan(sandbox.path())?;
    assert_eq!(plan.candidate_count(), 1);

    let second = authority(sandbox.path())?.execute(&plan)?;

    assert_eq!(first.receipt().generation().get(), 1);
    assert_eq!(second.receipt().generation().get(), 2);
    assert_complete(sandbox.path())?;
    sandbox.remove()?;
    Ok(())
}

#[test]
fn nothing_to_retire_and_a_stale_plan_refuse_before_any_intent() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-execute-refusals")?;
    let empty = plan(sandbox.path())?;
    let error = authority(sandbox.path())?
        .execute(&empty)
        .err()
        .ok_or("an empty plan executed")?;
    assert!(
        matches!(error, FilesystemGcError::NothingToRetire),
        "{error}"
    );

    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;
    let coordinates = observe(sandbox.path())?.coordinates();
    let exact = segment_receipt(coordinates, coordinates.catalog_digest())?;
    fs::write(disposition_path(sandbox.path()), &exact)?;
    let stale = plan(sandbox.path())?;
    assert_eq!(stale.candidate_count(), 1);
    fs::remove_file(disposition_path(sandbox.path()))?;
    let error = authority(sandbox.path())?
        .execute(&stale)
        .err()
        .ok_or("a stale plan executed")?;
    assert!(matches!(error, FilesystemGcError::PlanStale), "{error}");
    assert!(gc_entries(sandbox.path())?.is_empty());
    assert!(
        sandbox
            .path()
            .join("segments")
            .join(ORPHAN_SEGMENT_NAME)
            .exists()
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_reader_holding_the_fence_refuses_retirement_without_waiting() -> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-execute-readers")?;
    let plan = plan(sandbox.path())?;
    let reader = FilesystemRetentionSnapshot::load(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;

    let error = authority(sandbox.path())?
        .execute(&plan)
        .err()
        .ok_or("retirement ran beside a reader")?;

    assert!(matches!(error, FilesystemGcError::ReadersActive), "{error}");
    assert!(gc_entries(sandbox.path())?.is_empty());
    drop(reader);
    let _receipt = authority(sandbox.path())?.execute(&plan)?;
    assert_complete(sandbox.path())?;
    sandbox.remove()?;
    Ok(())
}

#[test]
fn every_interrupted_prefix_recovers_to_the_same_complete_state() -> Result<(), Box<dyn Error>> {
    for count in 0..=POINT_COUNT {
        let sandbox = disposed_store(&format!("gc-prefix-{count}"))?;
        interrupt(sandbox.path(), count)?;
        assert!(
            sandbox
                .path()
                .join("segments")
                .join(LIVE_SEGMENT_NAME)
                .exists(),
            "prefix {count} lost the live segment"
        );
        let plan_before = plan(sandbox.path());
        if count < 5 {
            let _plan = plan_before?;
        }

        let report = authority(sandbox.path())?.recover()?;

        assert_eq!(report.plan(), expected_plan(count), "prefix {count}");
        if report.plan() == GcRecoveryPlan::Idle {
            let _receipt = authority(sandbox.path())?.execute(&plan(sandbox.path())?)?;
        } else {
            assert!(report.receipt().is_some(), "prefix {count}");
        }
        assert_complete(sandbox.path())?;
        sandbox.remove()?;
    }
    Ok(())
}

#[test]
fn a_truncated_stage_is_discarded_and_the_retirement_then_completes() -> Result<(), Box<dyn Error>>
{
    for (count, stage, name) in [
        (0, GcFixedStage::Intent, "intent.next"),
        (8, GcFixedStage::Receipt, "receipt.next"),
    ] {
        let sandbox = disposed_store(&format!("gc-truncated-{count}"))?;
        interrupt(sandbox.path(), count)?;
        fs::write(sandbox.path().join("gc").join(name), [0xAB; 100])?;

        let report = authority(sandbox.path())?.recover()?;
        assert_eq!(report.plan(), GcRecoveryPlan::DiscardStage { stage });
        assert!(!sandbox.path().join("gc").join(name).exists());

        let report = authority(sandbox.path())?.recover()?;
        assert_eq!(report.plan(), expected_plan(count));
        if report.plan() == GcRecoveryPlan::Idle {
            let _receipt = authority(sandbox.path())?.execute(&plan(sandbox.path())?)?;
        }
        assert_complete(sandbox.path())?;
        sandbox.remove()?;
    }
    Ok(())
}

#[test]
fn a_durable_intent_excludes_retention_publication_and_another_retirement()
-> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-intent-excludes")?;
    let plan = plan(sandbox.path())?;
    interrupt(sandbox.path(), 6)?;

    let error = authority(sandbox.path())?
        .execute(&plan)
        .err()
        .ok_or("a retirement started over a durable intent")?;
    assert!(
        matches!(error, FilesystemGcError::RecoveryRequired { .. }),
        "{error}"
    );

    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut retention = reopen_authority(sandbox.path())?;
    let error = execute_retention_publication(&mut retention, &preparation)
        .err()
        .ok_or("retention published over a durable GC intent")?;
    let refused = match &error {
        RetentionPublicationError::CurrentVerification { source } => source
            .get_ref()
            .and_then(|refusal| refusal.downcast_ref::<RetentionCurrentStateRefusal>()),
        _ => None,
    };
    assert!(
        matches!(
            refused,
            Some(RetentionCurrentStateRefusal::GcIntentRetained)
        ),
        "{error}"
    );
    drop(retention);

    let report = authority(sandbox.path())?.recover()?;
    assert_eq!(report.plan(), expected_plan(6));
    assert_complete(sandbox.path())?;
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_absent_candidate_after_a_present_one_is_ambiguous_and_touches_nothing()
-> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-ambiguous")?;
    interrupt(sandbox.path(), 6)?;
    // The one candidate vanishes by other means, then a receipt stage
    // appears while a foreign entry keeps the pool from being explained.
    fs::write(sandbox.path().join("gc").join("receipt.next"), [0xAB; 320])?;

    let error = authority(sandbox.path())?
        .recover()
        .err()
        .ok_or("ambiguous residue recovered")?;

    assert!(matches!(error, FilesystemGcError::Ambiguity(_)), "{error}");
    assert!(sandbox.path().join("gc").join("receipt.next").exists());
    assert!(
        sandbox
            .path()
            .join("segments")
            .join(ORPHAN_SEGMENT_NAME)
            .exists()
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn admission_refuses_a_foreign_gc_entry_or_a_directory_in_place_of_a_record()
-> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-admission")?;
    fs::write(sandbox.path().join("gc").join("notes"), b"x")?;
    assert!(authority(sandbox.path()).is_err());
    fs::remove_file(sandbox.path().join("gc").join("notes"))?;
    fs::create_dir(sandbox.path().join("gc").join("intent"))?;
    assert!(authority(sandbox.path()).is_err());
    fs::remove_dir(sandbox.path().join("gc").join("intent"))?;
    let _authority = authority(sandbox.path())?;
    sandbox.remove()?;
    Ok(())
}
