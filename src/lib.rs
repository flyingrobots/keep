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
//! generations, and reconstruction anchors are validated. Retention
//! publication, recovery, and garbage collection remain intentionally absent.

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
    BlobIdBinaryParseError, BlobIdTextParseError, CanonicalCatalog, CanonicalLayoutRecord,
    CanonicalPublicationHead, CatalogAdmissionError, CatalogAllocationPhase, CatalogDecodeError,
    CatalogEncodeError, CatalogEntryDecodeError, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationOutcome, CatalogPublicationPhase,
    CatalogPublicationReadiness, CatalogPublicationReceipt, CatalogPublicationStorage,
    CatalogRestartArtifact, CatalogRestartByteLimit, CatalogRestartByteLimitError,
    CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy, CatalogSnapshot,
    CatalogSnapshotError, CatalogSuccessor, CatalogTransitionError, ChecksummedCatalog,
    ChecksummedPublicationHead, ChecksummedSegmentRecord, ClosedSegment,
    FilesystemCatalogPublicationError, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemPlatformAdmission, FilesystemPlatformAdmissionError,
    FilesystemRecoveryInventoryReader, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoverySegmentResumeOpenError,
    FilesystemRecoverySegmentResumer, FilesystemRecoverySegmentStage,
    FilesystemRecoveryStageCompleter, FilesystemRecoveryStageCompletionOpenError,
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder,
    FilesystemRecoveryStageError, FilesystemSegmentStage, FilesystemWriterLock, LayoutDecodeError,
    LayoutDecodePolicy, LayoutEncodeError, LayoutIdBinaryParseError, LayoutIdTextParseError,
    OpenedReusableSegment, PublicationHeadDecodeError, RecoveryCatalogStage,
    RecoveryCatalogStageError, RecoveryEntryName, RecoveryEntryNameError, RecoveryEntryRole,
    RecoveryInventory, RecoveryInventoryEntry, RecoveryInventoryError, RecoveryInventoryLimit,
    RecoveryInventoryLimitError, RecoveryInventoryOperation, RecoveryInventoryStorage,
    RecoveryNameClassificationError, RecoveryNameManifest, RecoveryNamedEntry, RecoveryNamespace,
    RecoveryNextHeadFinalizationError, RecoveryNextHeadFinalizationOutcome,
    RecoveryNextHeadFinalizationPlanError, RecoveryNextHeadFinalizationReadiness,
    RecoveryNextHeadFinalizationReceipt, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationStorage, RecoveryNextHeadFinalizationStorageError,
    RecoveryNextHeadFinalizationTarget, RecoveryNextHeadStage, RecoveryNextHeadStageError,
    RecoveryPoolNameError, RecoveryRequiredEntry, RecoverySegmentResumeError,
    RecoverySegmentResumePlanError, RecoverySegmentResumeRequest, RecoverySegmentResumeStorage,
    RecoverySegmentResumeStorageError, RecoverySegmentStage, RecoverySegmentStageError,
    RecoverySegmentTruncation, RecoveryStage, RecoveryStageAssessment,
    RecoveryStageAssessmentError, RecoveryStageByteAdmissionError, RecoveryStageCompletionError,
    RecoveryStageCompletionPlanError, RecoveryStageCompletionPool, RecoveryStageCompletionReceipt,
    RecoveryStageCompletionRequest, RecoveryStageCompletionStorage,
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
    StoreInitializationError, StoreInitializationPhase, StoreInitializationReceipt,
    StoreInitializationStorage, WriterLockAcquireError, WriterLockAcquirePhase,
    admit_recovery_stage_bytes, assess_recovery_stage, classify_recovery_catalog_stage,
    classify_recovery_names, classify_recovery_next_head_stage, classify_recovery_segment_stage,
    execute_recovery_next_head_finalization, execute_recovery_segment_resume,
    execute_recovery_stage_completion, execute_recovery_stage_discard, fingerprint_recovery_stage,
    initialize_store, plan_recovery_next_head_finalization, plan_recovery_segment_resume,
    plan_recovery_stage_completion, plan_recovery_stage_discard, publish_catalog_generation,
    read_recovery_inventory,
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
    LivenessGeneration, LivenessGenerationError, RetentionAnchor, RetentionNamespace,
    RetentionNamespaceDigest, RetentionNamespaceError, RootGeneration, RootGenerationError,
};
