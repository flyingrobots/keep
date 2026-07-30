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
//! and the complete synchronization mask. Live inventory and root revalidation,
//! filesystem migration, retention execution, recovery, and garbage collection
//! remain intentionally absent.

#[cfg(test)]
extern crate self as keep;

mod adapters;
mod blob;
mod catalog;
mod chunk;
mod layout;
mod profile;
mod reference;
mod retention;

#[cfg(feature = "repository-tasks")]
#[doc(hidden)]
pub use adapters::RepositoryInitializationStorage;
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
    FilesystemPlatformAdmission, FilesystemPlatformAdmissionError,
    FilesystemRecoveryInventoryReader, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoverySegmentResumeOpenError,
    FilesystemRecoverySegmentResumer, FilesystemRecoverySegmentStage,
    FilesystemRecoveryStageCompleter, FilesystemRecoveryStageCompletionOpenError,
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder,
    FilesystemRecoveryStageError, FilesystemSegmentStage, FilesystemWriterLock,
    ImmutablePoolInventoryDigest, InitialGcStateDigest, InitialRetentionStateDigest,
    LayoutDecodeError, LayoutDecodePolicy, LayoutEncodeError, LayoutIdBinaryParseError,
    LayoutIdTextParseError, MigrationSynchronizationMask, OpenedReusableSegment,
    PublicationHeadDecodeError, RecoveryCatalogStage, RecoveryCatalogStageError, RecoveryEntryName,
    RecoveryEntryNameError, RecoveryEntryRole, RecoveryInventory, RecoveryInventoryEntry,
    RecoveryInventoryError, RecoveryInventoryLimit, RecoveryInventoryLimitError,
    RecoveryInventoryOperation, RecoveryInventoryStorage, RecoveryNameClassificationError,
    RecoveryNameManifest, RecoveryNamedEntry, RecoveryNamespace, RecoveryNextHeadFinalizationError,
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
    StoreInitializationReceipt, StoreInitializationStorage, StoreMigrationIntentDecodeError,
    StoreMigrationIntentDigest, StoreMigrationInventoryEntry, StoreMigrationInventoryEntryCount,
    StoreMigrationInventoryEntryCountError, StoreMigrationInventoryError,
    StoreMigrationInventoryHasher, StoreMigrationPhase, StoreMigrationReceiptDecodeError,
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity, WriterLockAcquireError,
    WriterLockAcquirePhase, admit_recovery_stage_bytes, assess_recovery_stage,
    classify_recovery_catalog_stage, classify_recovery_names, classify_recovery_next_head_stage,
    classify_recovery_segment_stage, execute_recovery_next_head_finalization,
    execute_recovery_segment_resume, execute_recovery_stage_completion,
    execute_recovery_stage_discard, fingerprint_recovery_stage, initialize_store,
    plan_recovery_next_head_finalization, plan_recovery_segment_resume,
    plan_recovery_stage_completion, plan_recovery_stage_discard, publish_catalog_generation,
    read_recovery_inventory,
};
pub use adapters::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, CanonicalRetentionRoot, ChecksummedRetentionHead,
    PreparedRetentionPublication, RetentionClosureVerificationError, RetentionHeadDecodeError,
    RetentionManifestDecodeError, RetentionManifestEncodeError, RetentionNamespaceAdmission,
    RetentionPublicationError, RetentionPublicationOutcome, RetentionPublicationPhase,
    RetentionPublicationPreparation, RetentionPublicationPreparationError,
    RetentionPublicationReceipt, RetentionPublicationStorage, RetentionRootDecodeError,
    RetentionRootEncodeError, RetentionTransitionDisposition, RetentionTransitionError,
    RetentionTransitionPreflight, RetentionTransitionPreflightError, RetentionTransitionReadiness,
    VerifiedRetentionClosure, execute_retention_publication, plan_retention_transition,
    preflight_retention_transition, prepare_retention_publication, verify_retention_closure,
};
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
pub use layout::{
    AdmittedLayout, LayoutEntry, LayoutEntryLimit, LayoutEntryLimitError, LayoutId,
    LayoutIdMismatch, LayoutRecordLength, LayoutValidationError, RangePlan, RangePlanError,
};
pub use profile::{RegisteredStorageProfile, StorageProfileAdmissionError, StorageProfileId};
pub use reference::{
    IngestionAllocation, IngestionError, ProfileBoundary, PublishError, PublishedBlob,
    RangeReadError, RangeReadReceipt, ReconstructionError, ReconstructionReceipt, ReferenceStore,
    ReferenceStoreCapacity, StagedBlob,
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
