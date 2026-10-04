#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]
#![allow(
    clippy::multiple_crate_versions,
    reason = "the audited capability dependencies retain documented platform-only version overlap"
)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte and physical chunk identity,
//! deterministic streaming chunk detection, canonical flat-layout identity
//! and codecs, a capacity-bounded non-durable reference CAS, and explicit
//! immutable-segment writing and verified reading, canonical catalog
//! generations, platform-gated filesystem publication mechanics, bounded
//! immutable restart snapshots, typed store-initialization orchestration, and
//! production initialization for the admitted Linux ext4 profile. Recovery
//! inventory, name classification, bounded stage fingerprinting, exact
//! truncated-stage discard, and complete-stage valid-orphan recovery are
//! explicit. Exact next-head finalization now has a storage-independent
//! contract and a pinned writer-authorized filesystem adapter. Reusable-stage
//! continuation has a storage-independent planning and execution boundary plus
//! a pinned writer-authorized filesystem adapter. Core retention namespaces,
//! generations, realization policy, reconstruction anchors, and semantic roots
//! are validated; canonical in-memory root, manifest, and head encoding and
//! decoding, storage-independent expected-state transition planning,
//! deterministic bounded closure verification against a pinned catalog, and a
//! combined transition preflight proof and exact publication phase vocabulary
//! with a blocking storage capability port are available. Storage-independent
//! preparation binds preflight to exact canonical manifest and head successors.
//! Ordered publication revalidates authority, executes all durability phases,
//! and returns a complete receipt. The exact version-2 store-format marker has
//! canonical encoding, registered-definition admission, checksum verification,
//! and domain-separated identity. Migration-intent admission validates its
//! framing, checksum, catalog and predecessor grammar, registered definition,
//! deterministic store identity, and typed recovery coordinates. Completion
//! receipts bind an admitted intent and marker, registered empty-state digests,
//! and the complete synchronization mask. Writer-locked filesystem authority
//! now executes one fresh forward migration through exact fixed-record and
//! namespace transitions while retaining version-1 immutable bytes.
//! Partial-prefix recovery now plans and resumes lawful migration residue,
//! returning typed refusals and an ordered execution receipt. Filesystem
//! retention publication and complete-stage restart recovery are available.
//!
//! Incomplete retention stages require explicit disposition before recovery
//! effects; automatic disposal remains deferred. Execution failures preserve
//! their typed cause and distinguish known effects from uncertain effects or
//! durability. Writer authority coordinates cooperating writers in a managed
//! namespace; it does not isolate arbitrary out-of-band filesystem mutation.
//!
//! [`FilesystemRetentionSnapshot`] binds a catalog, retention head and manifest
//! under a shared reader fence and verifies selected roots on demand.
//! [`DurableStore`] composes that fenced view with authenticated reconstruction
//! and exact-range reads under explicit snapshot allocation limits.
//! [`VerificationReport`] names the subject, requested depth and established
//! evidence; unsupported depths refuse without downgrade or a partial report.
//!
//! Production durable ingestion, garbage collection and compaction remain
//! unimplemented here. General version-2 catalog publication still needs a
//! retained-closure gate; retention publication's live closure verification
//! does not establish that an arbitrary successor catalog preserves every root.

#[cfg(test)]
extern crate self as keep;

mod adapters;
mod authenticated_read;
mod blob;
mod catalog;
mod chunk;
mod gc;
mod layout;
mod profile;
mod reference;
mod retention;
mod segment_digest;
mod store;
mod verification;

pub use adapters::{
    AdmittedCatalog, AdmittedRecoveryStageBytes, AdmittedSegment, AdmittedSegmentRecord,
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    BlobIdBinaryParseError, BlobIdTextParseError, CanonicalCatalog, CanonicalLayoutRecord,
    CanonicalPublicationHead, CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent,
    CanonicalStoreMigrationReceipt, CatalogAdmissionError, CatalogAllocationPhase,
    CatalogDecodeError, CatalogEncodeError, CatalogEntryDecodeError, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationOutcome, CatalogPublicationPhase,
    CatalogPublicationReadiness, CatalogPublicationReceipt, CatalogPublicationStorage,
    CatalogRestartArtifact, CatalogRestartByteLimit, CatalogRestartByteLimitError,
    CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy, CatalogSnapshot,
    CatalogSnapshotError, CatalogSuccessor, CatalogTransitionError, ChecksummedCatalog,
    ChecksummedPublicationHead, ChecksummedSegmentRecord, ClosedSegment, EmptyDispositionSetDigest,
    FilesystemCatalogPublicationError, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemMigrationAuthorityArtifact, FilesystemMigrationAuthorityError,
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    FilesystemPlatformAdmission, FilesystemPlatformAdmissionError,
    FilesystemRecoveryInventoryReader, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoverySegmentResumeOpenError,
    FilesystemRecoverySegmentResumer, FilesystemRecoverySegmentStage,
    FilesystemRecoveryStageCompleter, FilesystemRecoveryStageCompletionOpenError,
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder,
    FilesystemRecoveryStageError, FilesystemSegmentStage, FilesystemStoreMigrationAuthority,
    FilesystemStoreMigrationInventoryReader, FilesystemVersionTwoAdmission, FilesystemWriterLock,
    ImmutablePoolInventoryDigest, InitialGcStateDigest, InitialRetentionStateDigest,
    LayoutDecodeError, LayoutDecodePolicy, LayoutEncodeError, LayoutIdBinaryParseError,
    LayoutIdTextParseError, MigrationInventoryNamespace, MigrationInventoryPool,
    MigrationSynchronizationMask, OpenedReusableSegment, PublicationHeadDecodeError,
    RecoveryCatalogStage, RecoveryCatalogStageError, RecoveryEntryName, RecoveryEntryNameError,
    RecoveryEntryRole, RecoveryInventory, RecoveryInventoryEntry, RecoveryInventoryError,
    RecoveryInventoryLimit, RecoveryInventoryLimitError, RecoveryInventoryOperation,
    RecoveryInventoryStorage, RecoveryNameClassificationError, RecoveryNameManifest,
    RecoveryNamedEntry, RecoveryNamespace, RecoveryNextHeadFinalizationError,
    RecoveryNextHeadFinalizationOutcome, RecoveryNextHeadFinalizationPlanError,
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationReceipt,
    RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationStorage,
    RecoveryNextHeadFinalizationStorageError, RecoveryNextHeadFinalizationTarget,
    RecoveryNextHeadStage, RecoveryNextHeadStageError, RecoveryPoolNameError,
    RecoveryRequiredEntry, RecoverySegmentResumeError, RecoverySegmentResumePlanError,
    RecoverySegmentResumeRequest, RecoverySegmentResumeStorage, RecoverySegmentResumeStorageError,
    RecoverySegmentStage, RecoverySegmentStageError, RecoverySegmentTruncation, RecoveryStage,
    RecoveryStageAssessment, RecoveryStageAssessmentError, RecoveryStageByteAdmissionError,
    RecoveryStageCompletionError, RecoveryStageCompletionPlanError, RecoveryStageCompletionPool,
    RecoveryStageCompletionReceipt, RecoveryStageCompletionRequest, RecoveryStageCompletionStorage,
    RecoveryStageCompletionStorageError, RecoveryStageCompletionTarget, RecoveryStageDiscardError,
    RecoveryStageDiscardOutcome, RecoveryStageDiscardPlanError, RecoveryStageDiscardReason,
    RecoveryStageDiscardReceipt, RecoveryStageDiscardRequest, RecoveryStageDiscardStorage,
    RecoveryStageDiscardStorageError, RecoveryStageEvidence, RecoveryStageFingerprint,
    RecoveryStageFingerprintAlgorithm, RecoveryStageFingerprintError, RecoveryStageLength,
    RecoveryStageMetadata, RecoveryStageMetadataError, RecoveryStageNamespacePhase,
    RecoveryStageParent, RecoveryStagePoolOutcome, RecoveryStageSynchronizationOutcome,
    ReusableRecoverySegment, SealedSegment, SegmentDigest, SegmentDurabilityPhase, SegmentHeader,
    SegmentHeaderError, SegmentPublication, SegmentPublicationError, SegmentReadError,
    SegmentReadPolicy, SegmentRecordAdmissionError, SegmentRecordChecksum,
    SegmentRecordDecodeError, SegmentRecordHeader, SegmentRecordHeaderError, SegmentRecordIdentity,
    SegmentRecordLength, SegmentRecordLimit, SegmentRecordLimitError, SegmentRecordPayloadLength,
    SegmentRecords, SegmentSeal, SegmentSealError, SegmentStage, SegmentStageCreateError,
    SegmentWriteError, SegmentWritePhase, StagedSegment, StorageProfileIdParseError,
    StoreFormatDefinitionDigest, StoreFormatMarkerDecodeError, StoreFormatMarkerDigest,
    StoreIdentifier, StoreInitializationError, StoreInitializationPhase,
    StoreInitializationReceipt, StoreInitializationStorage, StoreMigrationError,
    StoreMigrationIntentDecodeError, StoreMigrationIntentDigest, StoreMigrationInventoryEntry,
    StoreMigrationInventoryEntryCount, StoreMigrationInventoryEntryCountError,
    StoreMigrationInventoryError, StoreMigrationInventoryHasher, StoreMigrationPhase,
    StoreMigrationReceiptDecodeError, StoreMigrationStorage, StoreRootDeviceIdentity,
    StoreRootFileIdentity, StoreRootIdentityCoordinate, StoreRootMountIdentity,
    VersionTwoRecordRefusal, WriterLockAcquireError, WriterLockAcquirePhase,
    admit_recovery_stage_bytes, assess_recovery_stage, classify_recovery_catalog_stage,
    classify_recovery_names, classify_recovery_next_head_stage, classify_recovery_segment_stage,
    execute_recovery_next_head_finalization, execute_recovery_segment_resume,
    execute_recovery_stage_completion, execute_recovery_stage_discard, execute_store_migration,
    fingerprint_recovery_stage, initialize_store, plan_recovery_next_head_finalization,
    plan_recovery_segment_resume, plan_recovery_stage_completion, plan_recovery_stage_discard,
    publish_catalog_generation, read_recovery_inventory,
};
pub use adapters::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, AdmittedRecoveryDispositionReceipt,
    ArtifactContentDigest, ArtifactIdentityDigest, CanonicalGcRetirementIntent,
    CanonicalGcRetirementReceipt, CanonicalRecoveryDispositionReceipt, CatalogSuccessorProofDigest,
    DecisionEvidenceDigest, DispositionSetDigest, FilesystemGcAuthority, FilesystemGcError,
    GcCandidate, GcCandidateSetDigest, GcExecutionError, GcExecutionPhase, GcExecutionPoint,
    GcExecutionReceipt, GcExecutionStorage, GcFixedStage, GcIntentDerivationError,
    GcIntentEvidence, GcLimits, GcLimitsError, GcLivenessCoordinates, GcLivenessObservationError,
    GcLivenessSnapshot, GcLivenessSnapshotError, GcPlan, GcPlanAmbiguity, GcPlanError,
    GcPlannedCandidate, GcPlannedSegment, GcRecoveryAmbiguity, GcRecoveryPlan, GcRecoveryReport,
    GcResidue, GcRetainedClosure, GcRetentionState, GcRetirementIntent,
    GcRetirementIntentCoordinates, GcRetirementIntentDecodeError, GcRetirementIntentDigest,
    GcRetirementIntentEncodeError, GcRetirementIntentError, GcRetirementReceipt,
    GcRetirementReceiptDecodeError, GcSegmentClassification, GcUnreachableEvidence,
    ObservedHeadChecksum, PoolStateDigest, PreparedGcExecution, ReaderLockCoordinate,
    ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount, RecoveryArtifactKind,
    RecoveryClassification, RecoveryDispositionArtifact, RecoveryDispositionCoordinates,
    RecoveryDispositionDecision, RecoveryDispositionDecodeError, RecoveryDispositionField,
    RecoveryDispositionReceipt, SegmentPoolIdentityDigest, VerificationEvidenceDigest,
    catalog_successor_proof, derive_gc_intent, disposition_set_digest, execute_gc, is_gc_complete,
    observe_gc_liveness, plan_gc, plan_gc_recovery, post_retirement_pool_state,
    resume_gc_execution, segment_pool_identity,
};
pub use adapters::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, CanonicalRetentionRoot, ChecksummedRetentionHead,
    FilesystemRetentionAuthorityError, FilesystemRetentionDispositionError,
    FilesystemRetentionPublicationAuthority, FilesystemRetentionRecoveryError,
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, ObservedRetentionState,
    PreparedRetentionPublication, ReaderAttemptLimit, RecoveryDispositionAmbiguity,
    RecoveryDispositionError, RecoveryDispositionExecutionReceipt, RecoveryDispositionPhase,
    RecoveryDispositionPlan, RecoveryDispositionRefusal, RecoveryDispositionRequest,
    RecoveryDispositionStorage, RecoveryDispositionTarget, RetentionAuthorityDirectory,
    RetentionClosureVerificationError, RetentionCurrentStateRefusal, RetentionEffectDurability,
    RetentionFixedStage, RetentionHeadDecodeError, RetentionHeadStageAssessment,
    RetentionKnownEffect, RetentionManifestDecodeError, RetentionManifestEncodeError,
    RetentionManifestStageAssessment, RetentionNamespaceAdmission, RetentionNamespaceEffect,
    RetentionPool, RetentionPoolEntryObservation, RetentionPoolObservations,
    RetentionPublicationError, RetentionPublicationOutcome, RetentionPublicationPhase,
    RetentionPublicationPreparation, RetentionPublicationPreparationError,
    RetentionPublicationReceipt, RetentionPublicationStorage, RetentionRecordRefusal,
    RetentionRecoveryError, RetentionRecoveryEvidence, RetentionRecoveryOutcome,
    RetentionRecoveryPlan, RetentionRecoveryReceipt, RetentionRecoveryRefusal,
    RetentionRecoveryStep, RetentionRecoveryStorage, RetentionRootDecodeError,
    RetentionRootEncodeError, RetentionRootStageAssessment, RetentionSelectedRootRefusal,
    RetentionStageAssessment, RetentionStageAssessments, RetentionStorageBoundary,
    RetentionStorageError, RetentionStorageProgress, RetentionTransitionDisposition,
    RetentionTransitionError, RetentionTransitionPreflight, RetentionTransitionPreflightError,
    RetentionTransitionReadiness, RetentionViewCoordinates, RetentionViewError,
    RetentionViewSource, VerifiedRetentionClosure, assess_head_stage, assess_manifest_stage,
    assess_root_stage, collect_retention_view, collect_verification_view,
    execute_recovery_disposition, execute_retention_publication, execute_retention_recovery,
    plan_recovery_disposition, plan_retention_recovery, plan_retention_transition,
    preflight_retention_transition, prepare_retention_publication, resume_recovery_disposition,
    verify_retention_closure,
};
pub use adapters::{
    CancellationFlag, CancellationSignal, CopyError, CopyReceipt, NeverCancelled, StreamConsumer,
    TransferBounds, TransferError, TransferReceipt, TransferSegment, TransferSink, TransferSource,
    TransferSourceError, TransferWindow, WriteSink, WriteSinkError, copy_layout, transfer_blob,
    transfer_layout, transfer_layout_range, transfer_range,
};
pub use adapters::{
    CanonicalVerificationReceipt, ReceiptCorruption, ReceiptEvidenceKind, ReceiptMissing,
    ReceiptOutcomeKind, ReceiptRefusal, ReceiptRefusalClass, ReceiptSubjectKind, ReceiptViewKind,
    VERIFICATION_CONTRACT_VERSION, VerificationOutcome, VerificationReceipt,
    VerificationReceiptDecodeError, VerificationReceiptField, VerificationView,
};
pub use adapters::{
    CompactionObservation, CompactionPlan, CompactionPublish, CompactionReceipt,
    CompactionRecovery, CompactionRefusal, CompactionSegmentDisposition,
    FilesystemCompactionAuthority, FilesystemCompactionError, FilesystemCompactionRecoveryError,
    observe_compaction, plan_compaction, recover_compaction,
};
pub use adapters::{
    DurableIngestionError, DurableIngestionReceipt, DurableOutcome, DurableRangeReadReceipt,
    DurableReadError, DurableReconstructionReceipt, DurableSnapshot, DurableStagedBlob,
    DurableStore, DurableStoreError, DurableView, DurableWriter, IngestionAccounting,
    recover_durable_ingestion,
};
pub use adapters::{
    FilesystemMigrationRecoveryRefusal, FilesystemMigrationResidueKind, MIGRATION_NAMESPACE_PREFIX,
    StoreMigrationEffect, StoreMigrationFixedStage, StoreMigrationNamespacePrefix,
    StoreMigrationRecoveryAmbiguity, StoreMigrationRecoveryError, StoreMigrationRecoveryPlan,
    StoreMigrationRecoveryReceipt, StoreMigrationRecoveryStorage, StoreMigrationResidue,
    StoreMigrationStageDecodeError, plan_store_migration_recovery, recover_store_migration,
};
#[cfg(feature = "repository-tasks")]
#[doc(hidden)]
pub use adapters::{
    ObservedSegmentStage, RepositoryInitializationStorage, SegmentStageDurabilityEvent,
    SegmentStageObserver,
};
pub use adapters::{VerificationError, VerificationSource, verify_catalog_bytes, verify_segment};
pub use blob::{
    BlobHashError, BlobHasher, BlobId, BlobLength, BlobReadError, ByteLength, ByteOffset,
    ByteRange, ByteRangeError,
};
pub use catalog::{
    CatalogDigest, CatalogGeneration, CatalogGenerationError, CatalogLength, CatalogLengthError,
};
pub use chunk::{
    ChunkHashError, ChunkId, ChunkLength, ChunkOffset, ChunkSpan, ChunkingError, FastCdc,
};
pub use gc::{GcGeneration, GcGenerationError};
pub use layout::{
    AdmittedLayout, LayoutEntry, LayoutEntryLimit, LayoutEntryLimitError, LayoutId,
    LayoutIdMismatch, LayoutRecordLength, LayoutValidationError, RangePlan, RangePlanError,
};
pub use profile::{
    ProfileBoundary, RegisteredStorageProfile, StorageProfileAdmissionError, StorageProfileId,
};
pub use reference::{
    IngestionAllocation, IngestionError, PublishError, PublishedBlob, ReferenceStagedContent,
    ReferenceStore, ReferenceStoreCapacity, StagedBlob,
};
pub use retention::{
    LivenessGeneration, LivenessGenerationError, RegisteredRetentionProfile, RetentionAnchor,
    RetentionAnchorSetDigest, RetentionClosureCounter, RetentionClosureDigest,
    RetentionClosureLimit, RetentionClosureLimitError, RetentionClosureLimits,
    RetentionClosureUsage, RetentionGenerationExpectation, RetentionHead, RetentionHeadError,
    RetentionManifest, RetentionManifestDigest, RetentionManifestEntry, RetentionManifestError,
    RetentionManifestLength, RetentionManifestLengthError, RetentionNamespace,
    RetentionNamespaceDigest, RetentionNamespaceError, RetentionPolicy,
    RetentionProfileAdmissionError, RetentionRoot, RetentionRootDigest, RetentionRootError,
    RootGeneration, RootGenerationError,
};
pub use store::{
    CommitReceipt, ContentReads, ContentStaging, StagedByteLimit, StagedContent, StagingLimits,
};
pub use verification::{
    VerificationDepth, VerificationObservation, VerificationRefusal, VerificationReport,
    VerificationSubject, VerifiedSubject,
};

pub use adapters::{RangeReadError, ReconstructionError};
pub use authenticated_read::{RangeReadReceipt, ReconstructionReceipt};

pub use adapters::{FilesystemEntryKind, FilesystemNamespaceRefusal};
pub use verification::{CorruptionEvidence, MissingEvidence};

pub use reference::{
    ReferenceVerificationContext, ReferenceVerificationEvidence, ReferenceVerificationSource,
};
pub use verification::ReferenceVerificationDetails;

pub use adapters::{ReceiptSubject, ReceiptVerificationDepth, VerificationReceiptProjectionError};
