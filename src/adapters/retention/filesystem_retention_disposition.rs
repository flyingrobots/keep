//! This module owns filesystem execution of one explicit disposition under
//! writer authority and the exclusive reader fence.

use std::error::Error;
use std::fmt;
use std::io::{self, Read};

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_current::{self, read_exact_optional};
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_recovery::RetentionRecoveryContext;
use super::filesystem_retention_recovery_observation::RetentionRecoveryObservation;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, FilesystemRetentionRecoveryError,
    ReaderFence, RecoveryDispositionError, RecoveryDispositionPhase, RecoveryDispositionPlan,
    RecoveryDispositionRefusal, RecoveryDispositionRequest, RecoveryDispositionStorage,
    RecoveryDispositionTarget, RetentionRecoveryStorage, plan_recovery_disposition,
    plan_retention_recovery, resume_recovery_disposition,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;
use crate::adapters::{
    AdmittedRecoveryDispositionReceipt, ArtifactIdentityDigest,
    CanonicalRecoveryDispositionReceipt, ChecksummedPublicationHead, DecisionEvidenceDigest,
    GcRetentionState, ObservedHeadChecksum, ReaderLockIdentity, RecoveryArtifactKind,
    RecoveryClassification, RecoveryDispositionArtifact, RecoveryDispositionCoordinates,
    RecoveryDispositionDecision, RecoveryDispositionReceipt, filesystem_platform_profile,
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
enum PoolEntry {
    Root { namespace: String, name: String },
    Manifest { name: String },
}

/// A located immutable pool entry with its complete bytes.
type LocatedEntry = (PoolEntry, Box<[u8]>);
/// A directory entry name with its complete bytes.
type NamedEntry = (String, Box<[u8]>);

/// What a planned disposition binds before its context opens.
struct DispositionInputs {
    plan: RecoveryDispositionPlan,
    receipt: CanonicalRecoveryDispositionReceipt,
    artifact: Box<[u8]>,
    pool: PoolEntry,
}

/// Everything one disposition run holds between phases.
pub(super) struct DispositionContext {
    target: RecoveryDispositionTarget,
    decision: RecoveryDispositionDecision,
    receipt: CanonicalRecoveryDispositionReceipt,
    artifact: Box<[u8]>,
    pool: PoolEntry,
    name: String,
    recovery: Dir,
    dispositions: Dir,
    stage: Option<FilesystemRetentionStage>,
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
        let context = self.disposition_context(inputs, fence).map_err(observe)?;
        let from = resume_phase(&context)?;
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
            .ok_or_else(|| invalid_data("publication head is absent"))?;
        let head = ChecksummedPublicationHead::decode(&head_bytes).map_err(invalid_data_from)?;
        let checksum: [u8; 32] = head_bytes
            .get(HEAD_CHECKSUM_OFFSET..HEAD_LENGTH)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| invalid_data("publication head checksum slot"))?;
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
            reader_lock: ReaderLockIdentity::new(device, mount, file),
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
    context: &DispositionContext,
) -> Result<RecoveryDispositionPhase, FilesystemRetentionDispositionError> {
    let expected = context.receipt.encoded();
    let stage = read_bounded(&context.recovery, pool_name::DISPOSITION_STAGE).map_err(observe)?;
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
    match (stage.as_deref(), receipt.is_some()) {
        (None, false) => Ok(RecoveryDispositionPhase::WriteStage),
        (Some(bytes), false) if bytes == expected => Ok(RecoveryDispositionPhase::SynchronizeStage),
        (Some(bytes), false) if expected.starts_with(bytes) => {
            discard_stage(&context.recovery).map_err(observe)?;
            Ok(RecoveryDispositionPhase::WriteStage)
        }
        (Some(bytes), true) if bytes == expected => Ok(RecoveryDispositionPhase::RemoveStage),
        (None, true) => Ok(RecoveryDispositionPhase::RemoveRetainedStage),
        (Some(_), _) => Err(FilesystemRetentionDispositionError::Ambiguity(
            RecoveryDispositionAmbiguity::StageDiffers,
        )),
    }
}

const fn observe(source: io::Error) -> FilesystemRetentionDispositionError {
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

/// The retained stage's exact bytes and the pool entry recovery linked them to.
fn disposed_artifact(
    observation: &RetentionRecoveryObservation,
    target: RecoveryDispositionTarget,
) -> io::Result<(Box<[u8]>, PoolEntry)> {
    match target {
        RecoveryDispositionTarget::Root => {
            let bytes = observation
                .root()
                .ok_or_else(|| invalid_data("disposition target root stage vanished"))?
                .bytes
                .clone();
            let root = AdmittedRetentionRoot::decode(&bytes).map_err(invalid_data_from)?;
            let pool = PoolEntry::Root {
                namespace: pool_name::namespace(root.root().namespace().digest()),
                name: pool_name::root(root.root().generation(), root.digest()),
            };
            Ok((bytes, pool))
        }
        RecoveryDispositionTarget::Manifest => {
            let bytes = observation
                .manifest()
                .ok_or_else(|| invalid_data("disposition target manifest stage vanished"))?
                .bytes
                .clone();
            let manifest = AdmittedRetentionManifest::decode(&bytes).map_err(invalid_data_from)?;
            let pool = PoolEntry::Manifest {
                name: pool_name::manifest(manifest.manifest().generation(), manifest.digest()),
            };
            Ok((bytes, pool))
        }
    }
}

/// The artifact's pool-name digest: the identity the receipt is filed under.
fn pool_identity(
    pool: &PoolEntry,
    artifact: &[u8],
) -> Result<(RecoveryArtifactKind, ArtifactIdentityDigest), FilesystemRetentionDispositionError> {
    match pool {
        PoolEntry::Root { .. } => {
            let root = AdmittedRetentionRoot::decode(artifact)
                .map_err(|source| observe(invalid_data_from(source)))?;
            Ok((
                RecoveryArtifactKind::RetentionRoot,
                ArtifactIdentityDigest::new(*root.digest().as_bytes()),
            ))
        }
        PoolEntry::Manifest { .. } => {
            let manifest = AdmittedRetentionManifest::decode(artifact)
                .map_err(|source| observe(invalid_data_from(source)))?;
            Ok((
                RecoveryArtifactKind::RetentionManifest,
                ArtifactIdentityDigest::new(*manifest.digest().as_bytes()),
            ))
        }
    }
}

fn receipt_for(
    artifact: &[u8],
    (kind, identity): (RecoveryArtifactKind, ArtifactIdentityDigest),
    decision: RecoveryDispositionDecision,
    coordinates: RecoveryDispositionCoordinates,
) -> io::Result<CanonicalRecoveryDispositionReceipt> {
    let disposed = RecoveryDispositionArtifact {
        kind,
        classification: RecoveryClassification::CompleteOrphan,
        length: u64::try_from(artifact.len()).map_err(invalid_data_from)?,
        identity_digest: identity,
        content_digest: CanonicalRecoveryDispositionReceipt::artifact_content_digest(artifact),
    };
    let evidence = DecisionEvidenceDigest::new(trailing_checksum(artifact)?);
    Ok(CanonicalRecoveryDispositionReceipt::from_receipt(
        &RecoveryDispositionReceipt::new(disposed, decision, coordinates, evidence),
    ))
}

/// The artifact record's trailing checksum: the evidence the decision was
/// made over.
fn trailing_checksum(bytes: &[u8]) -> io::Result<[u8; 32]> {
    let start = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_data("artifact is shorter than its checksum"))?;
    bytes
        .get(start..)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| invalid_data("artifact checksum slot"))
}

fn invalid_data_from(error: impl Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// Reads `recovery/disposition.next` up to one byte past the receipt length.
fn read_bounded(recovery: &Dir, name: &str) -> io::Result<Option<Vec<u8>>> {
    let mut file = match exact_record::open_read(recovery, name) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source),
    };
    if !file.metadata()?.is_file() {
        return Err(invalid_data("disposition stage is not a regular file"));
    }
    let mut bytes = Vec::new();
    let limit = u64::try_from(RECEIPT_LENGTH)
        .map_err(invalid_data_from)?
        .saturating_add(1);
    file.by_ref().take(limit).read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

/// The first regular entry of `directory` whose name ends with `suffix`,
/// with its complete bytes.
fn entry_with_suffix(directory: &Dir, suffix: &str) -> io::Result<Option<NamedEntry>> {
    for entry in directory.entries()? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if !name.ends_with(suffix) {
            continue;
        }
        let metadata = directory.symlink_metadata(&name)?;
        if !metadata.is_file() {
            return Err(invalid_data("retention pool entry is not a regular file"));
        }
        let length = usize::try_from(metadata.len()).map_err(invalid_data_from)?;
        let bytes = read_exact_optional(directory, &name, length)?
            .ok_or_else(|| invalid_data("retention pool entry vanished"))?;
        return Ok(Some((name, bytes)));
    }
    Ok(None)
}

fn discard_stage(recovery: &Dir) -> io::Result<()> {
    recovery.remove_file(pool_name::DISPOSITION_STAGE)?;
    exact_record::require_absent(recovery, pool_name::DISPOSITION_STAGE)
        .map_err(|_source| invalid_data("discarded disposition stage remained visible"))?;
    synchronize_directory(recovery)
}

fn no_disposition() -> io::Error {
    invalid_data("no disposition is in progress")
}

/// Removes `name` from `directory` after proving it still holds `expected`.
fn unlink_verified(directory: &Dir, name: &str, expected: &[u8]) -> io::Result<()> {
    let observed = read_exact_optional(directory, name, expected.len())?
        .ok_or_else(|| invalid_data("retired pool entry is already absent"))?;
    if observed.as_ref() != expected {
        return Err(invalid_data(
            "retired pool entry bytes disagree with the receipt",
        ));
    }
    directory.remove_file(name)?;
    exact_record::require_absent(directory, name)
        .map_err(|_source| invalid_data("retired pool entry remained visible"))
}

impl RecoveryDispositionStorage for FilesystemRetentionPublicationAuthority {
    fn write_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        context.stage = Some(FilesystemRetentionStage::create(
            &context.recovery,
            pool_name::DISPOSITION_STAGE,
            context.receipt.encoded(),
        )?);
        Ok(())
    }

    fn synchronize_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
            )?);
        }
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage.synchronize(&context.recovery)
    }

    fn link_disposition_receipt(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage.link(&context.recovery, &context.dispositions, &context.name)
    }

    fn synchronize_dispositions(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.dispositions)
    }

    fn remove_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
            )?);
        }
        let stage = context.stage.take().ok_or_else(no_disposition)?;
        stage.remove(&context.recovery, &context.dispositions, &context.name)
    }

    fn synchronize_recovery(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.recovery)
    }

    fn remove_retained_stage(&mut self) -> io::Result<()> {
        let target = self.disposition.as_ref().ok_or_else(no_disposition)?.target;
        match target {
            RecoveryDispositionTarget::Root => RetentionRecoveryStorage::remove_root_stage(self),
            RecoveryDispositionTarget::Manifest => {
                RetentionRecoveryStorage::remove_manifest_stage(self)
            }
        }
    }

    fn synchronize_retention_after_disposition(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)
    }

    fn remove_pool_entry(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match &context.pool {
            PoolEntry::Root { namespace, name } => {
                let directory = self.roots.open_dir_nofollow(namespace)?;
                unlink_verified(&directory, name, &context.artifact)?;
                synchronize_directory(&directory)?;
                if directory.entries()?.next().is_none() {
                    drop(directory);
                    self.roots.remove_dir(namespace)?;
                }
                Ok(())
            }
            PoolEntry::Manifest { name } => {
                unlink_verified(&self.manifests, name, &context.artifact)
            }
        }
    }

    fn synchronize_pool(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match context.pool {
            PoolEntry::Root { .. } => synchronize_directory(&self.roots),
            PoolEntry::Manifest { .. } => synchronize_directory(&self.manifests),
        }
    }
}
