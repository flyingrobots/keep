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
//! inventory, name classification, and bounded stage fingerprinting are
//! read-only; semantic recovery planning, execution, retention, and garbage
//! collection APIs remain intentionally absent until their contracts have
//! executable specifications.

#[cfg(test)]
extern crate self as keep;

mod adapters;
mod blob;
mod catalog;
mod chunk;
mod layout;
mod profile;
mod reference;

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
    FilesystemPlatformAdmission, FilesystemRecoveryInventoryReader, FilesystemRecoveryStageError,
    FilesystemSegmentStage, FilesystemWriterLock, LayoutDecodeError, LayoutDecodePolicy,
    LayoutEncodeError, LayoutIdBinaryParseError, LayoutIdTextParseError,
    PublicationHeadDecodeError, RecoveryCatalogStage, RecoveryCatalogStageError, RecoveryEntryName,
    RecoveryEntryNameError, RecoveryEntryRole, RecoveryInventory, RecoveryInventoryEntry,
    RecoveryInventoryError, RecoveryInventoryLimit, RecoveryInventoryLimitError,
    RecoveryInventoryOperation, RecoveryInventoryStorage, RecoveryNameClassificationError,
    RecoveryNameManifest, RecoveryNamedEntry, RecoveryNamespace, RecoveryNextHeadStage,
    RecoveryNextHeadStageError, RecoveryPoolNameError, RecoveryRequiredEntry, RecoverySegmentStage,
    RecoverySegmentStageError, RecoverySegmentTruncation, RecoveryStage, RecoveryStageAssessment,
    RecoveryStageAssessmentError, RecoveryStageByteAdmissionError, RecoveryStageEvidence,
    RecoveryStageFingerprint, RecoveryStageFingerprintAlgorithm, RecoveryStageFingerprintError,
    RecoveryStageLength, RecoveryStageMetadata, RecoveryStageMetadataError,
    RecoveryStageNamespacePhase, ReusableRecoverySegment, SealedSegment, SegmentDigest,
    SegmentDurabilityPhase, SegmentHeader, SegmentHeaderError, SegmentPublication,
    SegmentPublicationError, SegmentReadError, SegmentReadPolicy, SegmentRecordAdmissionError,
    SegmentRecordChecksum, SegmentRecordDecodeError, SegmentRecordHeader, SegmentRecordHeaderError,
    SegmentRecordIdentity, SegmentRecordLength, SegmentRecordLimit, SegmentRecordLimitError,
    SegmentRecordPayloadLength, SegmentRecords, SegmentSeal, SegmentSealError, SegmentStage,
    SegmentStageCreateError, SegmentWriteError, SegmentWritePhase, StagedSegment,
    StorageProfileIdParseError, StoreInitializationError, StoreInitializationPhase,
    StoreInitializationReceipt, StoreInitializationStorage, WriterLockAcquireError,
    WriterLockAcquirePhase, admit_recovery_stage_bytes, assess_recovery_stage,
    classify_recovery_catalog_stage, classify_recovery_names, classify_recovery_next_head_stage,
    classify_recovery_segment_stage, fingerprint_recovery_stage, initialize_store,
    publish_catalog_generation, read_recovery_inventory,
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
