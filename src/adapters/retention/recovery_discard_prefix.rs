//! This module owns earlier-evidence admission before incomplete-stage discard.

use super::{
    RetentionFixedStage as Fixed, RetentionPoolEntryObservation as Pool, RetentionRecoveryEvidence,
    RetentionRecoveryRefusal as Refusal, RetentionStageAssessment as Stage,
};

pub(super) fn admit(evidence: &RetentionRecoveryEvidence<'_, '_>) -> Result<(), Refusal> {
    let stages = evidence.stages();
    let pools = evidence.pools();
    let later = if matches!(stages.head, Stage::Truncated { .. }) {
        Fixed::Head
    } else if matches!(stages.manifest, Stage::Truncated { .. }) {
        Fixed::Manifest
    } else {
        return Ok(());
    };
    if !matches!(stages.root, Stage::Complete(_)) || pools.root != Pool::Identical {
        return Err(Refusal::TruncatedStageWithoutEarlierEvidence {
            stage: later,
            earlier_stage: Fixed::Root,
        });
    }
    if later == Fixed::Head
        && (!matches!(stages.manifest, Stage::Complete(_)) || pools.manifest != Pool::Identical)
    {
        return Err(Refusal::TruncatedStageWithoutEarlierEvidence {
            stage: later,
            earlier_stage: Fixed::Manifest,
        });
    }
    Ok(())
}
