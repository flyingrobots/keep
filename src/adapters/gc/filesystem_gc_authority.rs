//! This module owns exact writer-locked, reader-fenced filesystem GC
//! execution and recovery over one admitted version-two root.

use std::path::{Path, PathBuf};

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_gc_error::observe;
use super::filesystem_gc_residue::{self as residue, INTENT_STAGE, RECEIPT_STAGE};
use super::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, CanonicalGcRetirementIntent,
    CanonicalGcRetirementReceipt, FilesystemGcError as Error, GcExecutionPhase, GcExecutionPoint,
    GcFixedStage, GcIntentEvidence, GcLimits, GcPlan, GcRecoveryPlan, GcRetirementReceipt,
    ReaderLockIdentity, derive_gc_intent, observe_gc_liveness, plan_gc, plan_gc_recovery,
    resume_gc_execution,
};
use crate::adapters::retention::{FilesystemRetentionStage, ReaderFence};
use crate::adapters::{
    CatalogRestartPolicy, FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission,
    FilesystemWriterLock, ReaderAttemptLimit, filesystem_platform_profile,
};
use crate::{GcGeneration, RegisteredRetentionProfile};

const GC: &str = "gc";
const SEGMENTS: &str = "segments";

/// Everything one retirement holds between phases.
pub(super) struct GcExecutionContext {
    pub(super) intent: CanonicalGcRetirementIntent,
    pub(super) receipt: Option<CanonicalGcRetirementReceipt>,
    pub(super) intent_stage: Option<FilesystemRetentionStage>,
    pub(super) receipt_stage: Option<FilesystemRetentionStage>,
    _fence: ReaderFence,
}

/// Proof that [`FilesystemGcAuthority::prepare`] bound an intent and the
/// exclusive fence, with the count execution iterates over.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedGcExecution {
    candidate_count: usize,
}

impl PreparedGcExecution {
    /// The number of candidates the bound intent names.
    #[must_use]
    pub const fn candidate_count(self) -> usize {
        self.candidate_count
    }
}

/// What one recovery run found and did.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcRecoveryReport {
    plan: GcRecoveryPlan,
    receipt: Option<GcRetirementReceipt>,
}

impl GcRecoveryReport {
    /// The plan the residue admitted.
    pub const fn plan(&self) -> GcRecoveryPlan {
        self.plan
    }

    /// The complete receipt, when the residue was or became complete.
    pub const fn receipt(&self) -> Option<&GcRetirementReceipt> {
        self.receipt.as_ref()
    }
}

/// Exclusive authority to retire released segments from one pinned root.
///
/// The authority holds the writer lock for its lifetime and the exclusive
/// reader fence for each retirement. Passed to
/// [`execute_gc`](super::execute_gc) after [`Self::prepare`], its
/// [`GcExecutionStorage`](super::GcExecutionStorage) implementation runs
/// the fixed-stage protocol of `gc.md`; [`Self::execute`] does both.
/// [`Self::recover`] resolves whatever residue a prior run left.
#[must_use]
pub struct FilesystemGcAuthority {
    pub(super) root: Dir,
    pub(super) gc: Dir,
    pub(super) segments: Dir,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) context: Option<GcExecutionContext>,
    store_root: PathBuf,
    _lock: FilesystemWriterLock,
}

impl FilesystemGcAuthority {
    /// Pins one admitted version-two root for GC.
    ///
    /// `store_root` must be the path `admission` was reopened from; the
    /// liveness view re-admits it on every retirement. The constructor
    /// mutates nothing.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemGcError::Observe`](Error::Observe) when a pinned
    /// directory cannot be opened.
    pub fn open(
        admission: FilesystemVersionTwoAdmission,
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, Error> {
        let (lock, _retention, _roots, _manifests) = admission.into_parts();
        let root = lock.clone_directory().map_err(observe)?;
        let gc = root.open_dir_nofollow(GC).map_err(observe)?;
        let segments = root.open_dir_nofollow(SEGMENTS).map_err(observe)?;
        Ok(Self {
            root,
            gc,
            segments,
            policy,
            context: None,
            store_root: store_root.to_path_buf(),
            _lock: lock,
        })
    }

    /// Resolves the residue of an interrupted retirement.
    ///
    /// Idle and complete residue change nothing; a truncated stage written
    /// before any authority is discarded; every other lawful residue resumes
    /// execution at its documented point under the exclusive fence. Any
    /// other residue is a typed ambiguity and nothing is touched.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemGcError`] at the exact observation, ambiguity,
    /// fence, or phase refusal.
    pub fn recover(&mut self) -> Result<GcRecoveryReport, Error> {
        self.context = None;
        let residue = residue::read(&self.gc, &self.segments).map_err(observe)?;
        let plan = plan_gc_recovery(&residue).map_err(Error::Ambiguity)?;
        let receipt = match plan {
            GcRecoveryPlan::Idle => None,
            GcRecoveryPlan::Complete => residue
                .receipt
                .as_deref()
                .map(AdmittedGcRetirementReceipt::decode_unbound)
                .transpose()
                .map_err(|source| observe(residue::invalid_from(source)))?,
            GcRecoveryPlan::DiscardStage { stage } => {
                let name = match stage {
                    GcFixedStage::Intent => INTENT_STAGE,
                    GcFixedStage::Receipt => RECEIPT_STAGE,
                };
                residue::discard(&self.gc, name).map_err(observe)?;
                None
            }
            GcRecoveryPlan::Resume { from } => {
                let bytes = if from.phase == GcExecutionPhase::SynchronizeIntentStage {
                    residue.intent_stage.as_deref()
                } else {
                    residue.intent.as_deref()
                };
                let admitted = AdmittedGcRetirementIntent::decode(
                    bytes.ok_or_else(|| observe(residue::invalid("GC intent vanished")))?,
                )
                .map_err(|source| observe(residue::invalid_from(source)))?;
                let intent = CanonicalGcRetirementIntent::from_intent(admitted.intent())
                    .map_err(Error::Encode)?;
                let count = admitted.intent().candidates().len();
                Some(self.run(intent, count, from)?.receipt().to_owned())
            }
        };
        Ok(GcRecoveryReport { plan, receipt })
    }

    /// Re-proves `plan` against the reopened store under writer authority
    /// and the exclusive reader fence, derives its one canonical intent, and
    /// binds both for execution.
    ///
    /// The intent's generation succeeds the prior receipt's, or is one.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemGcError`] when `gc` holds residue, readers hold
    /// the fence, the plan names nothing, the re-observed store plans
    /// differently, or the intent refuses.
    pub fn prepare(&mut self, plan: &GcPlan) -> Result<PreparedGcExecution, Error> {
        self.context = None;
        let residue = residue::read(&self.gc, &self.segments).map_err(observe)?;
        let generation = match plan_gc_recovery(&residue).map_err(Error::Ambiguity)? {
            GcRecoveryPlan::Idle => GcGeneration::new(1).map_err(Error::Generation)?,
            GcRecoveryPlan::Complete => prior_generation(residue.receipt.as_deref())?
                .successor()
                .map_err(Error::Generation)?,
            plan => return Err(Error::RecoveryRequired { plan }),
        };
        if plan.candidate_count() == 0 {
            return Err(Error::NothingToRetire);
        }
        let fence = acquire_fence(&self.root)?;
        let snapshot = self.reobserve()?;
        if plan_gc(&snapshot, GcLimits::MAXIMUM).map_err(Error::Plan)? != *plan {
            return Err(Error::PlanStale);
        }
        let intent = derive_gc_intent(plan, &snapshot, self.evidence(generation, &fence)?)
            .map_err(Error::Intent)?;
        let intent = CanonicalGcRetirementIntent::from_intent(&intent).map_err(Error::Encode)?;
        let candidate_count = intent.intent().candidates().len();
        self.context = Some(GcExecutionContext {
            intent,
            receipt: None,
            intent_stage: None,
            receipt_stage: None,
            _fence: fence,
        });
        Ok(PreparedGcExecution { candidate_count })
    }

    /// Prepares and executes one complete retirement of `plan`.
    ///
    /// # Errors
    ///
    /// As [`Self::prepare`], then [`FilesystemGcError::Execute`](Error::Execute)
    /// at the refused phase; the completed phases' effects remain for
    /// [`Self::recover`].
    pub fn execute(&mut self, plan: &GcPlan) -> Result<CanonicalGcRetirementReceipt, Error> {
        let prepared = self.prepare(plan)?;
        let context = self.context.take().ok_or(Error::NothingToRetire)?;
        let intent = context.intent.clone();
        self.context = Some(context);
        self.run(intent, prepared.candidate_count, GcExecutionPoint::START)
    }

    /// The intent [`Self::prepare`] bound, until execution completes or
    /// the context is cleared.
    #[must_use]
    pub fn bound_intent(&self) -> Option<&CanonicalGcRetirementIntent> {
        self.context.as_ref().map(|context| &context.intent)
    }

    /// Returns the receipt of the retirement the authority last completed
    /// through an external driver, clearing its context.
    pub fn take_receipt(&mut self) -> Option<CanonicalGcRetirementReceipt> {
        self.context.take().and_then(|context| context.receipt)
    }

    fn run(
        &mut self,
        intent: CanonicalGcRetirementIntent,
        candidate_count: usize,
        from: GcExecutionPoint,
    ) -> Result<CanonicalGcRetirementReceipt, Error> {
        if self.context.is_none() {
            let fence = acquire_fence(&self.root)?;
            self.context = Some(GcExecutionContext {
                intent,
                receipt: None,
                intent_stage: None,
                receipt_stage: None,
                _fence: fence,
            });
        }
        let result = resume_gc_execution(self, candidate_count, from).map_err(Error::Execute);
        let receipt = self.context.take().and_then(|context| context.receipt);
        result.and_then(|_executed| receipt.ok_or(Error::NothingToRetire))
    }

    fn reobserve(&self) -> Result<super::GcLivenessSnapshot, Error> {
        let view = FilesystemRetentionSnapshot::load_under_writer_authority(
            &self.store_root,
            self.policy,
            ReaderAttemptLimit::DEFAULT,
        )
        .map_err(|source| Error::Snapshot(Box::new(source)))?;
        observe_gc_liveness(&self.store_root, &view, self.policy)
            .map_err(|source| Error::Liveness(Box::new(source)))
    }

    fn evidence(
        &self,
        generation: GcGeneration,
        fence: &ReaderFence,
    ) -> Result<GcIntentEvidence, Error> {
        let (device, file) = fence.identity().map_err(observe)?;
        let mount = filesystem_platform_profile::root_identity(&self.root)
            .map_err(observe)?
            .mount();
        Ok(GcIntentEvidence {
            generation,
            profile: RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            reader_lock: ReaderLockIdentity::new(device, mount, file),
        })
    }
}

fn prior_generation(receipt: Option<&[u8]>) -> Result<GcGeneration, Error> {
    let bytes = receipt.ok_or_else(|| observe(residue::invalid("GC receipt vanished")))?;
    AdmittedGcRetirementReceipt::decode_unbound(bytes)
        .map(|receipt| receipt.generation())
        .map_err(|source| observe(residue::invalid_from(source)))
}

fn acquire_fence(root: &Dir) -> Result<ReaderFence, Error> {
    ReaderFence::acquire_exclusive(root).map_err(|source| {
        if source.kind() == std::io::ErrorKind::WouldBlock {
            Error::ReadersActive
        } else {
            observe(source)
        }
    })
}
