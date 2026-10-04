//! This module owns GC caller preservation of retained intent-stage identity.

use super::{authority, disposed_store, gc_entries, plan};
use crate::adapters::gc::{
    GcExecutionPhase as Phase, GcExecutionPoint, GcExecutionStorage, resume_gc_execution,
};
use crate::adapters::retention::{
    RetentionRecordRefusal, RetentionStorageBoundary as Boundary, RetentionStorageError,
};
use std::{error::Error, fs, io};

// Size: medium. Oracle: byte-equal substitution cannot replace retained physical
// evidence; GC must preserve the shared stage's exact failure without rewrapping.
// Deterministic namespace substitution is a fault, not raw-writer isolation support.
// Delete only if stronger GC caller identity laws subsume this source/refusal check.
#[test]
fn gc_preserves_the_original_stage_refusal_before_linking() -> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-shared-stage-source")?;
    let plan = plan(sandbox.path())?;
    let mut writer = authority(sandbox.path())?;
    let prepared = writer.prepare(&plan)?;
    writer.write_intent_stage()?;
    let stage = sandbox.path().join("gc/intent.next");
    let retained = fs::read(&stage)?;
    fs::rename(&stage, sandbox.path().join("gc/original"))?;
    fs::write(&stage, &retained)?;
    let Err(error) = resume_gc_execution(
        &mut writer,
        prepared.candidate_count(),
        GcExecutionPoint::at(Phase::SynchronizeIntentStage),
    ) else {
        return Err("substituted source was admitted".into());
    };
    assert_eq!(
        error.point(),
        GcExecutionPoint::at(Phase::SynchronizeIntentStage)
    );
    assert!(
        error.executed().is_empty(),
        "nothing after substitution may run"
    );
    let progress = error
        .storage_progress()
        .ok_or("stage failure report lost")?;
    assert_eq!(progress.boundary(), Boundary::SourceVerification);
    assert!(progress.known_effects().is_empty());
    assert_eq!(progress.uncertain_effect(), None);
    let original = error
        .source()
        .and_then(|e| e.downcast_ref::<io::Error>())
        .and_then(io::Error::get_ref)
        .and_then(|e| e.downcast_ref::<RetentionStorageError>());
    assert!(
        matches!(original, Some(RetentionStorageError::Operation { source, .. }) if matches!(source.as_ref(), RetentionStorageError::Refused { source: RetentionRecordRefusal::KindLengthOrIdentity })),
        "original shared-stage source must not be nested or erased"
    );
    assert_eq!(gc_entries(sandbox.path())?, ["intent.next", "original"]);
    assert_eq!(fs::read(stage)?, retained);
    assert_eq!(fs::read(sandbox.path().join("gc/original"))?, retained);
    Ok(())
}
