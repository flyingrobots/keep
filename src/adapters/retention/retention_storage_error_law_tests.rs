//! This module owns recovery executor preservation of typed port refusals.

use std::error::Error;

use super::{
    RetentionRecordRefusal as Refusal, RetentionRecoveryOutcome, RetentionRecoveryPlan,
    RetentionRecoveryStep as Step, RetentionRecoveryStorage, RetentionStorageError as StorageError,
    execute_retention_recovery,
};

struct Refusing(Refusal);

impl Refusing {
    fn refuse(&self) -> Result<(), StorageError> {
        Err(StorageError::Refused { source: self.0 })
    }
}

impl RetentionRecoveryStorage for Refusing {
    fn discard_head_stage(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn discard_manifest_stage(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn discard_root_stage(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn link_root(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn link_manifest(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn finalize_head(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn remove_root_stage(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
    fn remove_manifest_stage(&mut self) -> Result<(), StorageError> {
        self.refuse()
    }
}

// Size: small. Oracle: the public executor preserves a port's precise refusal
// for every capability; this law makes no filesystem fault-coverage claim.
// Delete when the port is removed or stronger cheaper propagation evidence subsumes it.
#[test]
fn every_recovery_capability_preserves_its_supplied_record_refusal() -> Result<(), Box<dyn Error>> {
    let steps = [
        Step::DiscardHeadStage,
        Step::DiscardManifestStage,
        Step::DiscardRootStage,
        Step::LinkRoot,
        Step::LinkManifest,
        Step::FinalizeHead,
        Step::RemoveRootStage,
        Step::RemoveManifestStage,
    ];
    let refusals = [
        Refusal::LengthOverflow,
        Refusal::KindOrLength,
        Refusal::KindLengthOrIdentity,
        Refusal::Bytes,
        Refusal::TrailingBytes,
        Refusal::RemainedVisible,
    ];
    for step in steps {
        for refusal in refusals {
            let plan = RetentionRecoveryPlan::new(vec![step], RetentionRecoveryOutcome::Committed);
            let error = execute_retention_recovery(&mut Refusing(refusal), &plan)
                .err()
                .ok_or("refused storage was reported as committed")?;
            assert!(
                matches!(error.storage_error(), StorageError::Refused { source } if *source == refusal),
                "{step:?} must preserve {refusal:?}: {error:?}"
            );
        }
    }
    Ok(())
}
