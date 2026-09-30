//! This module owns pure planning of retention recovery from restart evidence.

use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    ObservedRetentionState, RetentionFixedStage, RetentionPool,
    RetentionPoolEntryObservation as Pool, RetentionPoolObservations, RetentionRecoveryEvidence,
    RetentionRecoveryOutcome, RetentionRecoveryPlan, RetentionRecoveryRefusal as Refusal,
    RetentionRecoveryStep as Step, RetentionStageAssessment as Stage,
};
use crate::{LivenessGeneration, RootGeneration};

type Current<'state> = Option<&'state ObservedRetentionState>;

/// Plans retention recovery from complete restart evidence.
///
/// The call performs no I/O. It applies the documented classification: a
/// truncated stage with no later-ordered effect is discarded; a complete
/// root or manifest stage is linked into its pool and retained as a
/// recovery-protected orphan; a complete head stage naming the staged
/// manifest is finalized and both retained stages are removed; a staged
/// generation the published head already names is cleaned up. Any other
/// combination is unrecoverable ambiguity and refuses before any effect.
///
/// # Errors
///
/// Returns [`RetentionRecoveryRefusal`](super::RetentionRecoveryRefusal) naming
/// the exact ambiguity.
pub fn plan_retention_recovery(
    evidence: RetentionRecoveryEvidence<'_, '_>,
) -> Result<RetentionRecoveryPlan, Refusal> {
    let head_present = evidence.stages().head.is_present();
    let manifest_present = evidence.stages().manifest.is_present();
    let (current, stages, pools) = evidence.into_parts();
    let mut steps = Vec::new();
    let head = match stages.head {
        Stage::Absent => None,
        Stage::Truncated { .. } => {
            steps.push(Step::DiscardHeadStage);
            None
        }
        Stage::Corrupt(source) => return Err(Refusal::corrupt_head(source)),
        Stage::Complete(head) => Some(head),
    };
    let manifest = match stages.manifest {
        Stage::Absent => None,
        Stage::Truncated { .. } => {
            if head_present || pools.manifest != Pool::Absent {
                return Err(Refusal::TruncatedStageWithLaterEffect {
                    stage: RetentionFixedStage::Manifest,
                });
            }
            steps.push(Step::DiscardManifestStage);
            None
        }
        Stage::Corrupt(source) => return Err(Refusal::corrupt_manifest(source)),
        Stage::Complete(manifest) => Some(manifest),
    };
    let root = match stages.root {
        Stage::Absent => None,
        Stage::Truncated { .. } => {
            if head_present || manifest_present || pools.root != Pool::Absent {
                return Err(Refusal::TruncatedStageWithLaterEffect {
                    stage: RetentionFixedStage::Root,
                });
            }
            steps.push(Step::DiscardRootStage);
            None
        }
        Stage::Corrupt(source) => return Err(Refusal::corrupt_root(source)),
        Stage::Complete(root) => Some(root),
    };
    if root.is_some() && pools.root == Pool::Different {
        return Err(Refusal::PoolEntryDiffers {
            pool: RetentionPool::Roots,
        });
    }
    if manifest.is_some() && pools.manifest == Pool::Different {
        return Err(Refusal::PoolEntryDiffers {
            pool: RetentionPool::Manifests,
        });
    }
    match (head, manifest, root) {
        (Some(head), Some(manifest), Some(root)) => finalize_head(
            current,
            CompleteStages {
                head: &head,
                manifest: &manifest,
                root: &root,
            },
            pools,
            steps,
        ),
        (Some(_), _, _) => Err(Refusal::HeadStageWithoutManifestStage),
        (None, Some(manifest), root) => {
            plan_manifest(current, &manifest, root.as_ref(), pools, steps)
        }
        (None, None, Some(root)) => plan_root(current, &root, pools.root, steps),
        (None, None, None) => Ok(RetentionRecoveryPlan::new(
            steps,
            RetentionRecoveryOutcome::Clean,
        )),
    }
}

/// The three complete stages a head finalization is planned from.
#[derive(Clone, Copy)]
struct CompleteStages<'a, 'bytes> {
    head: &'a ChecksummedRetentionHead<'bytes>,
    manifest: &'a AdmittedRetentionManifest<'bytes>,
    root: &'a AdmittedRetentionRoot<'bytes>,
}

fn finalize_head(
    current: Current<'_>,
    stages: CompleteStages<'_, '_>,
    pools: RetentionPoolObservations,
    mut steps: Vec<Step>,
) -> Result<RetentionRecoveryPlan, Refusal> {
    let CompleteStages {
        head,
        manifest,
        root,
    } = stages;
    let head = head.head();
    if head.manifest_digest() != manifest.digest()
        || head.generation() != manifest.manifest().generation()
    {
        return Err(Refusal::HeadStageNamesOtherManifest);
    }
    if !manifest_names_root(manifest, root) {
        return Err(Refusal::ManifestStageNamesOtherRoot);
    }
    if pools.root != Pool::Identical {
        return Err(Refusal::RootNotLinkedBeforeHead);
    }
    if pools.manifest != Pool::Identical {
        return Err(Refusal::ManifestNotLinkedBeforeHead);
    }
    if !is_committed(current, manifest) && !manifest_succeeds(current, manifest) {
        return Err(Refusal::HeadPredecessorMismatch);
    }
    steps.extend([
        Step::FinalizeHead,
        Step::RemoveRootStage,
        Step::RemoveManifestStage,
    ]);
    Ok(RetentionRecoveryPlan::new(
        steps,
        RetentionRecoveryOutcome::Committed,
    ))
}

fn plan_manifest(
    current: Current<'_>,
    manifest: &AdmittedRetentionManifest<'_>,
    root: Option<&AdmittedRetentionRoot<'_>>,
    pools: RetentionPoolObservations,
    mut steps: Vec<Step>,
) -> Result<RetentionRecoveryPlan, Refusal> {
    if is_committed(current, manifest) {
        if let Some(root) = root {
            if !manifest_names_root(manifest, root) {
                return Err(Refusal::ManifestStageNamesOtherRoot);
            }
            if pools.root != Pool::Identical {
                return Err(Refusal::RootNotLinkedBeforeHead);
            }
            steps.push(Step::RemoveRootStage);
        }
        steps.push(Step::RemoveManifestStage);
        return Ok(RetentionRecoveryPlan::new(
            steps,
            RetentionRecoveryOutcome::Committed,
        ));
    }
    let root = root.ok_or(Refusal::ManifestStageWithoutRootStage)?;
    if !manifest_names_root(manifest, root) {
        return Err(Refusal::ManifestStageNamesOtherRoot);
    }
    if !manifest_succeeds(current, manifest) {
        return Err(Refusal::ManifestNotSuccessor);
    }
    if !root_succeeds(current, root) {
        return Err(Refusal::RootNotSuccessor);
    }
    if pools.root == Pool::Absent {
        steps.push(Step::LinkRoot);
    }
    if pools.manifest == Pool::Absent {
        steps.push(Step::LinkManifest);
    }
    Ok(RetentionRecoveryPlan::new(
        steps,
        RetentionRecoveryOutcome::Protected {
            root_stage: true,
            manifest_stage: true,
        },
    ))
}

fn plan_root(
    current: Current<'_>,
    root: &AdmittedRetentionRoot<'_>,
    root_pool: Pool,
    mut steps: Vec<Step>,
) -> Result<RetentionRecoveryPlan, Refusal> {
    if !root_succeeds(current, root) {
        return Err(Refusal::RootNotSuccessor);
    }
    if root_pool == Pool::Absent {
        steps.push(Step::LinkRoot);
    }
    Ok(RetentionRecoveryPlan::new(
        steps,
        RetentionRecoveryOutcome::Protected {
            root_stage: true,
            manifest_stage: false,
        },
    ))
}

/// Whether the published head already names the staged manifest.
fn is_committed(current: Current<'_>, manifest: &AdmittedRetentionManifest<'_>) -> bool {
    current.is_some_and(|current| {
        current.head().manifest_digest() == manifest.digest()
            && current.head().generation() == manifest.manifest().generation()
    })
}

fn manifest_names_root(
    manifest: &AdmittedRetentionManifest<'_>,
    root: &AdmittedRetentionRoot<'_>,
) -> bool {
    let namespace = root.root().namespace().digest();
    let entries = manifest.manifest().entries();
    entries
        .binary_search_by_key(&namespace, |entry| entry.namespace())
        .ok()
        .and_then(|index| entries.get(index))
        .is_some_and(|entry| {
            entry.root_generation() == root.root().generation()
                && entry.root_digest() == root.digest()
        })
}

fn manifest_succeeds(current: Current<'_>, manifest: &AdmittedRetentionManifest<'_>) -> bool {
    let manifest = manifest.manifest();
    current.map_or_else(
        || manifest.predecessor().is_none() && manifest.generation() == LivenessGeneration::INITIAL,
        |current| {
            manifest.predecessor() == Some(current.head().manifest_digest())
                && current
                    .head()
                    .generation()
                    .successor()
                    .is_ok_and(|successor| successor == manifest.generation())
        },
    )
}

fn root_succeeds(current: Current<'_>, root: &AdmittedRetentionRoot<'_>) -> bool {
    let namespace = root.root().namespace().digest();
    let entry = current.and_then(|current| {
        let entries = current.manifest().entries();
        entries
            .binary_search_by_key(&namespace, |entry| entry.namespace())
            .ok()
            .and_then(|index| entries.get(index).copied())
    });
    entry.map_or_else(
        || {
            root.root().predecessor().is_none()
                && root.root().generation() == RootGeneration::INITIAL
        },
        |entry| {
            root.root().predecessor() == Some(entry.root_digest())
                && entry
                    .root_generation()
                    .successor()
                    .is_ok_and(|successor| successor == root.root().generation())
        },
    )
}
