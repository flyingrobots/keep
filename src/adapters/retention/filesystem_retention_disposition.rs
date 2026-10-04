//! This module owns filesystem planning and resumption of one explicit
//! disposition under writer authority and the exclusive reader fence; the
//! phase effects live in `filesystem_retention_disposition_storage`.

use std::error::Error;
use std::fmt;
use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_current::{self, read_exact_optional};
use super::filesystem_retention_disposition_evidence::{
    discard_stage, disposed_artifact, entry_with_suffix, pool_identity, receipt_for,
};
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_recovery::RetentionRecoveryContext;
use super::filesystem_retention_recovery_observation::RetentionRecoveryObservation;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{
    FilesystemRetentionRecoveryError, ReaderFence, RecoveryDispositionError,
    RecoveryDispositionPhase, RecoveryDispositionPlan, RecoveryDispositionRefusal,
    RecoveryDispositionRequest, RecoveryDispositionTarget, plan_recovery_disposition,
    plan_retention_recovery, resume_recovery_disposition,
};
use crate::adapters::filesystem_stage_observation::StageObservation;
use crate::adapters::{
    AdmittedRecoveryDispositionReceipt, ArtifactIdentityDigest,
    CanonicalRecoveryDispositionReceipt, ChecksummedPublicationHead, GcRetentionState,
    ObservedHeadChecksum, ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount,
    RecoveryArtifactKind, RecoveryDispositionCoordinates, RecoveryDispositionDecision,
    filesystem_platform_profile,
};

const HEAD_NAME: &str = "HEAD";
const HEAD_LENGTH: usize = crate::adapters::publication_head_decoder::ENCODED_LENGTH;
const HEAD_CHECKSUM_OFFSET: usize = 96;
const RECEIPT_LENGTH: usize = 320;

/// Residue that admits no lawful resumption of a disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionAmbiguity {
    /// `recovery/disposition.next` holds bytes other than this disposition's
    /// canonical receipt.
    StageDiffers,
    /// The canonical receipt entry holds other bytes.
    ReceiptDiffers,
    /// A retired artifact's pool entry no longer carries the bytes its
    /// receipt names.
    PoolEntryDiffers,
}

/// Failure to dispose one protected stage.
#[derive(Debug)]
pub enum FilesystemRetentionDispositionError {
    /// Recovery, which runs first, refused.
    Recovery {
        /// The exact recovery failure.
        source: FilesystemRetentionRecoveryError,
    },
    /// The request cannot be planned over the recovered evidence.
    Plan {
        /// The exact refusal.
        source: RecoveryDispositionRefusal,
    },
    /// A reader holds the shared fence; disposition never waits on readers.
    ReadersActive,
    /// Observing the store's coordinates or residue refused.
    Observe {
        /// The original failure.
        source: io::Error,
    },
    /// The residue admits no lawful resumption.
    Ambiguity(RecoveryDispositionAmbiguity),
    /// A durable phase refused.
    Execute {
        /// The refused phase and its cause.
        source: RecoveryDispositionError,
    },
}

impl fmt::Display for FilesystemRetentionDispositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recovery { .. } => formatter.write_str("retention recovery refused"),
            Self::Plan { source } => write!(formatter, "disposition refused: {source}"),
            Self::ReadersActive => {
                formatter.write_str("a reader holds the fence; disposition does not wait")
            }
            Self::Observe { .. } => formatter.write_str("disposition observation refused"),
            Self::Ambiguity(ambiguity) => {
                write!(formatter, "disposition residue is ambiguous: {ambiguity:?}")
            }
            Self::Execute { source } => write!(formatter, "{source}"),
        }
    }
}

impl Error for FilesystemRetentionDispositionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Recovery { source } => Some(source),
            Self::Plan { source } => Some(source),
            Self::Observe { source } => Some(source),
            Self::Execute { source } => Some(source),
            Self::ReadersActive | Self::Ambiguity(_) => None,
        }
    }
}

/// Where a disposed artifact lives in its immutable pool.
pub(super) enum PoolEntry {
    Root { namespace: String, name: String },
    Manifest { name: String },
}

/// A located immutable pool entry with its complete bytes.
pub(super) type LocatedEntry = (PoolEntry, Box<[u8]>);

/// What a planned disposition binds before its context opens.
struct DispositionInputs {
    plan: RecoveryDispositionPlan,
    receipt: CanonicalRecoveryDispositionReceipt,
    artifact: Box<[u8]>,
    pool: PoolEntry,
}

/// Everything one disposition run holds between phases.
pub(super) struct DispositionContext {
    pub(super) target: RecoveryDispositionTarget,
    pub(super) decision: RecoveryDispositionDecision,
    pub(super) receipt: CanonicalRecoveryDispositionReceipt,
    pub(super) artifact: Box<[u8]>,
    pub(super) pool: PoolEntry,
    pub(super) name: String,
    pub(super) recovery: Dir,
    pub(super) dispositions: Dir,
    pub(super) stage: Option<FilesystemRetentionStage>,
    pub(super) stage_observation: Option<StageObservation>,
    _fence: ReaderFence,
}

impl FilesystemRetentionPublicationAuthority {
    /// Records one explicit finalize-or-retire decision over a
    /// recovery-protected stage and removes the stage, so publication may
    /// proceed.
    ///
    /// Recovery runs first, so a complete stage is linked before it can be
    /// disposed. The receipt is written through the fixed-stage protocol
    /// under the exclusive reader fence and is durable before the retained
    /// stage is removed. `Finalize` keeps the linked pool entry and needs a
    /// published retention head; `Retire` also unlinks the entry, because an
    /// absent head admits no pool artifact and no collector exists for the
    /// retention pools. An interrupted run resumes from its residue on the
    /// next call with the same request.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionDispositionError`] at the exact recovery,
    /// planning, fence, observation, residue, or phase refusal.
    pub fn dispose(
        &mut self,
        request: RecoveryDispositionRequest,
    ) -> Result<CanonicalRecoveryDispositionReceipt, FilesystemRetentionDispositionError> {
        self.attempt = None;
        let _recovered = self
            .recover()
            .map_err(|source| FilesystemRetentionDispositionError::Recovery { source })?;
        let observation =
            RetentionRecoveryObservation::observe(&self.retention, &self.roots, &self.manifests)
                .map_err(observe)?;
        let evidence = observation.evidence();
        let pools = evidence.pools();
        let head_published = evidence.current().is_some();
        let recovery = plan_retention_recovery(evidence).map_err(|source| {
            FilesystemRetentionDispositionError::Recovery {
                source: FilesystemRetentionRecoveryError::Plan { source },
            }
        })?;
        let plan = match plan_recovery_disposition(&recovery, pools, head_published, request) {
            Ok(plan) => plan,
            Err(
                refusal @ (RecoveryDispositionRefusal::NothingProtected
                | RecoveryDispositionRefusal::TargetNotRetained { .. }),
            ) => return self.finish_retired_pool_entry(request, refusal),
            Err(source) => return Err(FilesystemRetentionDispositionError::Plan { source }),
        };
        self.recovery =
            Some(RetentionRecoveryContext::reopen(&self.retention, &observation).map_err(observe)?);
        let fence = acquire_fence(&self.root)?;
        let (artifact, pool) = disposed_artifact(&observation, plan.target()).map_err(observe)?;
        let coordinates = self.disposition_coordinates(&fence).map_err(observe)?;
        let receipt = receipt_for(
            &artifact,
            pool_identity(&pool, &artifact)?,
            plan.decision(),
            coordinates,
        )
        .map_err(observe)?;
        let inputs = DispositionInputs {
            plan,
            receipt,
            artifact,
            pool,
        };
        let mut context = self.disposition_context(inputs, fence).map_err(observe)?;
        let from = resume_phase(&mut context)?;
        self.run_disposition(context, from)
    }

    /// Completes a retirement whose receipt is durable but whose pool entry
    /// still exists: the residue of process death after the retained stage
    /// was removed. Any other residue returns the planner's refusal.
    fn finish_retired_pool_entry(
        &mut self,
        request: RecoveryDispositionRequest,
        refusal: RecoveryDispositionRefusal,
    ) -> Result<CanonicalRecoveryDispositionReceipt, FilesystemRetentionDispositionError> {
        let recovery = self
            .root
            .open_dir_nofollow(pool_name::RECOVERY)
            .map_err(observe)?;
        let dispositions = recovery
            .open_dir_nofollow(pool_name::DISPOSITIONS)
            .map_err(observe)?;
        let fence = acquire_fence(&self.root)?;
        let coordinates = self.disposition_coordinates(&fence).map_err(observe)?;
        let kind = match request.target {
            RecoveryDispositionTarget::Root => RecoveryArtifactKind::RetentionRoot,
            RecoveryDispositionTarget::Manifest => RecoveryArtifactKind::RetentionManifest,
        };
        for entry in dispositions.entries().map_err(observe)? {
            let name = entry
                .map_err(observe)?
                .file_name()
                .to_string_lossy()
                .into_owned();
            let Some(bytes) =
                read_exact_optional(&dispositions, &name, RECEIPT_LENGTH).map_err(observe)?
            else {
                continue;
            };
            let admitted = AdmittedRecoveryDispositionReceipt::decode(&bytes)
                .map_err(|source| observe(invalid_data_from(source)))?;
            let semantic = *admitted.receipt();
            let exact = semantic.artifact().kind == kind
                && semantic.decision() == RecoveryDispositionDecision::Retire
                && semantic.coordinates() == coordinates;
            if !exact {
                continue;
            }
            let Some((pool, artifact)) = self
                .locate_pool_entry(request.target, semantic.artifact().identity_digest)
                .map_err(observe)?
            else {
                continue;
            };
            if CanonicalRecoveryDispositionReceipt::artifact_content_digest(&artifact)
                != semantic.artifact().content_digest
            {
                return Err(FilesystemRetentionDispositionError::Ambiguity(
                    RecoveryDispositionAmbiguity::PoolEntryDiffers,
                ));
            }
            let receipt = CanonicalRecoveryDispositionReceipt::from_receipt(&semantic);
            let context = DispositionContext {
                target: request.target,
                decision: RecoveryDispositionDecision::Retire,
                receipt,
                artifact,
                pool,
                name,
                recovery,
                dispositions,
                stage: None,
                stage_observation: None,
                _fence: fence,
            };
            return self.run_disposition(context, RecoveryDispositionPhase::RemovePoolEntry);
        }
        Err(FilesystemRetentionDispositionError::Plan { source: refusal })
    }

    fn run_disposition(
        &mut self,
        context: DispositionContext,
        from: RecoveryDispositionPhase,
    ) -> Result<CanonicalRecoveryDispositionReceipt, FilesystemRetentionDispositionError> {
        let receipt = context.receipt.clone();
        let decision = context.decision;
        self.disposition = Some(context);
        let result = resume_recovery_disposition(self, decision, from)
            .map_err(|source| FilesystemRetentionDispositionError::Execute { source });
        self.disposition = None;
        self.recovery = None;
        result.map(|_executed| receipt)
    }

    /// Finds the pool entry named by `identity` for `target`: a root under
    /// any namespace directory, or a manifest.
    fn locate_pool_entry(
        &self,
        target: RecoveryDispositionTarget,
        identity: ArtifactIdentityDigest,
    ) -> io::Result<Option<LocatedEntry>> {
        let suffix = format!(
            "-{}{}",
            crate::adapters::digest_hex::DigestHex(identity.as_bytes()),
            match target {
                RecoveryDispositionTarget::Root => pool_name::ROOT_SUFFIX,
                RecoveryDispositionTarget::Manifest => pool_name::MANIFEST_SUFFIX,
            }
        );
        match target {
            RecoveryDispositionTarget::Root => {
                for namespace in self.roots.entries()? {
                    let namespace = namespace?.file_name().to_string_lossy().into_owned();
                    let directory = self.roots.open_dir_nofollow(&namespace)?;
                    if let Some((name, bytes)) = entry_with_suffix(&directory, &suffix)? {
                        return Ok(Some((PoolEntry::Root { namespace, name }, bytes)));
                    }
                }
                Ok(None)
            }
            RecoveryDispositionTarget::Manifest => Ok(entry_with_suffix(&self.manifests, &suffix)?
                .map(|(name, bytes)| (PoolEntry::Manifest { name }, bytes))),
        }
    }

    /// Observes the publication head, catalog, retention state, and the
    /// locked `reader.lock` identity the decision is made under.
    fn disposition_coordinates(
        &self,
        fence: &ReaderFence,
    ) -> io::Result<RecoveryDispositionCoordinates> {
        let head_bytes = read_exact_optional(&self.root, HEAD_NAME, HEAD_LENGTH)?
            .ok_or_else(|| invalid_data(super::FilesystemRetentionStageRefusal::HeadAbsent))?;
        let head = ChecksummedPublicationHead::decode(&head_bytes).map_err(invalid_data_from)?;
        let checksum: [u8; 32] = head_bytes
            .get(HEAD_CHECKSUM_OFFSET..HEAD_LENGTH)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| {
                invalid_data(super::FilesystemRetentionStageRefusal::HeadChecksumSlot)
            })?;
        let retention = filesystem_retention_current::observe(&self.retention, &self.manifests)?
            .map_or(GcRetentionState::Empty, |current| {
                GcRetentionState::Published {
                    generation: current.head().generation(),
                    manifest_digest: current.head().manifest_digest(),
                }
            });
        let (device, file) = fence.identity()?;
        let mount = filesystem_platform_profile::root_identity(&self.root)?.mount();
        Ok(RecoveryDispositionCoordinates {
            publication_generation: head.generation(),
            publication_checksum: ObservedHeadChecksum::new(checksum),
            catalog_generation: head.generation(),
            catalog_digest: head.catalog_digest(),
            retention,
            reader_lock: ReaderLockIdentity::new(
                ReaderLockDevice::new(device),
                ReaderLockMount::new(mount),
                ReaderLockFile::new(file),
            ),
        })
    }

    fn disposition_context(
        &self,
        inputs: DispositionInputs,
        fence: ReaderFence,
    ) -> io::Result<DispositionContext> {
        let recovery = self.root.open_dir_nofollow(pool_name::RECOVERY)?;
        let dispositions = recovery.open_dir_nofollow(pool_name::DISPOSITIONS)?;
        let identity = inputs.receipt.receipt().artifact().identity_digest;
        Ok(DispositionContext {
            target: inputs.plan.target(),
            decision: inputs.plan.decision(),
            receipt: inputs.receipt,
            artifact: inputs.artifact,
            pool: inputs.pool,
            name: pool_name::disposition(identity.as_bytes()),
            recovery,
            dispositions,
            stage: None,
            stage_observation: None,
            _fence: fence,
        })
    }
}

/// Classifies the disposition residue and names the phase to resume at:
/// no stage and no receipt starts fresh; an exact stage resumes at its
/// synchronization; an exact stage beside the identical receipt resumes
/// at stage removal; the identical receipt alone resumes at the retained
/// retention stage. A truncated stage with no receipt is discarded first.
fn resume_phase(
    context: &mut DispositionContext,
) -> Result<RecoveryDispositionPhase, FilesystemRetentionDispositionError> {
    let expected = context.receipt.encoded();
    context.stage_observation = StageObservation::read(
        &context.recovery,
        pool_name::DISPOSITION_STAGE,
        RECEIPT_LENGTH + 1,
    )
    .map_err(observe)?;
    let stage = context
        .stage_observation
        .as_ref()
        .map(StageObservation::bytes);
    let receipt = read_exact_optional(&context.dispositions, &context.name, RECEIPT_LENGTH)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::InvalidData {
                FilesystemRetentionDispositionError::Ambiguity(
                    RecoveryDispositionAmbiguity::ReceiptDiffers,
                )
            } else {
                observe(source)
            }
        })?;
    if receipt.as_deref().is_some_and(|bytes| bytes != expected) {
        return Err(FilesystemRetentionDispositionError::Ambiguity(
            RecoveryDispositionAmbiguity::ReceiptDiffers,
        ));
    }
    match (stage, receipt.is_some()) {
        (None, false) => Ok(RecoveryDispositionPhase::WriteStage),
        (Some(bytes), false) if bytes == expected => Ok(RecoveryDispositionPhase::SynchronizeStage),
        (Some(bytes), false) if expected.starts_with(bytes) => {
            let observed = context.stage_observation.as_ref().ok_or_else(|| {
                observe(invalid_data(
                    super::FilesystemRetentionStageRefusal::DispositionStageObservationAbsent,
                ))
            })?;
            discard_stage(&context.recovery, observed).map_err(observe)?;
            context.stage_observation = None;
            Ok(RecoveryDispositionPhase::WriteStage)
        }
        (Some(bytes), true) if bytes == expected => Ok(RecoveryDispositionPhase::RemoveStage),
        (None, true) => Ok(RecoveryDispositionPhase::RemoveRetainedStage),
        (Some(_), _) => Err(FilesystemRetentionDispositionError::Ambiguity(
            RecoveryDispositionAmbiguity::StageDiffers,
        )),
    }
}

pub(super) const fn observe(source: io::Error) -> FilesystemRetentionDispositionError {
    FilesystemRetentionDispositionError::Observe { source }
}

fn acquire_fence(root: &Dir) -> Result<ReaderFence, FilesystemRetentionDispositionError> {
    ReaderFence::acquire_exclusive(root).map_err(|source| {
        if source.kind() == io::ErrorKind::WouldBlock {
            FilesystemRetentionDispositionError::ReadersActive
        } else {
            observe(source)
        }
    })
}

pub(super) fn invalid_data_from(error: impl Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
