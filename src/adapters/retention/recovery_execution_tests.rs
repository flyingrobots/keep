//! Retention recovery execution laws against a recording fake storage.

use std::error::Error;
use std::io;

use super::{
    RetentionRecoveryError, RetentionRecoveryOutcome, RetentionRecoveryPlan,
    RetentionRecoveryStep as Step, RetentionRecoveryStorage, execute_retention_recovery,
};

#[derive(Default)]
struct Recording {
    calls: Vec<Step>,
    refuse_at: Option<Step>,
}

impl Recording {
    fn record(&mut self, step: Step) -> io::Result<()> {
        if self.refuse_at == Some(step) {
            return Err(io::Error::other("injected refusal"));
        }
        self.calls.push(step);
        Ok(())
    }
}

impl RetentionRecoveryStorage for Recording {
    fn discard_head_stage(&mut self) -> io::Result<()> {
        self.record(Step::DiscardHeadStage)
    }
    fn discard_manifest_stage(&mut self) -> io::Result<()> {
        self.record(Step::DiscardManifestStage)
    }
    fn discard_root_stage(&mut self) -> io::Result<()> {
        self.record(Step::DiscardRootStage)
    }
    fn link_root(&mut self) -> io::Result<()> {
        self.record(Step::LinkRoot)
    }
    fn link_manifest(&mut self) -> io::Result<()> {
        self.record(Step::LinkManifest)
    }
    fn finalize_head(&mut self) -> io::Result<()> {
        self.record(Step::FinalizeHead)
    }
    fn remove_root_stage(&mut self) -> io::Result<()> {
        self.record(Step::RemoveRootStage)
    }
    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        self.record(Step::RemoveManifestStage)
    }
}

const FINALIZE: [Step; 3] = [
    Step::FinalizeHead,
    Step::RemoveRootStage,
    Step::RemoveManifestStage,
];

#[test]
fn every_step_calls_exactly_its_capability_in_plan_order() -> Result<(), Box<dyn Error>> {
    let all = [
        Step::DiscardHeadStage,
        Step::DiscardManifestStage,
        Step::DiscardRootStage,
        Step::LinkRoot,
        Step::LinkManifest,
        Step::FinalizeHead,
        Step::RemoveRootStage,
        Step::RemoveManifestStage,
    ];
    let plan = RetentionRecoveryPlan::new(all.to_vec(), RetentionRecoveryOutcome::Committed);
    let mut storage = Recording::default();

    let receipt = execute_retention_recovery(&mut storage, &plan)?;

    assert_eq!(storage.calls, all);
    assert_eq!(receipt.executed(), all);
    assert_eq!(receipt.outcome(), RetentionRecoveryOutcome::Committed);
    Ok(())
}

#[test]
fn an_empty_plan_touches_nothing_and_reports_its_outcome() -> Result<(), Box<dyn Error>> {
    let plan = RetentionRecoveryPlan::new(Vec::new(), RetentionRecoveryOutcome::Clean);
    let mut storage = Recording::default();

    let receipt = execute_retention_recovery(&mut storage, &plan)?;

    assert!(storage.calls.is_empty());
    assert!(receipt.executed().is_empty());
    assert_eq!(receipt.outcome(), RetentionRecoveryOutcome::Clean);
    Ok(())
}

#[test]
fn a_refused_step_stops_execution_and_names_the_completed_prefix() -> Result<(), Box<dyn Error>> {
    let plan = RetentionRecoveryPlan::new(FINALIZE.to_vec(), RetentionRecoveryOutcome::Committed);
    let mut storage = Recording {
        calls: Vec::new(),
        refuse_at: Some(Step::RemoveRootStage),
    };

    let error: RetentionRecoveryError = execute_retention_recovery(&mut storage, &plan)
        .err()
        .ok_or("an injected refusal was reported as success")?;

    assert_eq!(error.step(), Step::RemoveRootStage);
    assert_eq!(error.executed(), [Step::FinalizeHead]);
    assert_eq!(storage.calls, [Step::FinalizeHead]);
    assert!(error.source().is_some());
    Ok(())
}
