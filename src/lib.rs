#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte and physical chunk identity,
//! deterministic streaming chunk detection, canonical flat-layout identity
//! and codecs, a capacity-bounded non-durable reference CAS, and explicit
//! immutable-segment writing and verified reading. Durable namespace
//! publication, retention, and recovery APIs remain intentionally absent until
//! their contracts have executable specifications.

mod adapters;
mod blob;
mod catalog;
mod chunk;
mod layout;
mod profile;
mod reference;

pub use adapters::{
    AdmittedSegment, AdmittedSegmentRecord, BlobIdBinaryParseError, BlobIdTextParseError,
    CanonicalLayoutRecord, ChecksummedPublicationHead, ChecksummedSegmentRecord,
    FilesystemSegmentStage, LayoutDecodeError, LayoutDecodePolicy, LayoutEncodeError,
    LayoutIdBinaryParseError, LayoutIdTextParseError, PublicationHeadDecodeError, SealedSegment,
    SegmentDigest, SegmentDurabilityPhase, SegmentHeader, SegmentHeaderError, SegmentReadError,
    SegmentReadPolicy, SegmentRecordAdmissionError, SegmentRecordChecksum,
    SegmentRecordDecodeError, SegmentRecordHeader, SegmentRecordHeaderError, SegmentRecordIdentity,
    SegmentRecordLength, SegmentRecordLimit, SegmentRecordLimitError, SegmentRecordPayloadLength,
    SegmentRecords, SegmentSeal, SegmentSealError, SegmentStage, SegmentStageCreateError,
    SegmentWriteError, SegmentWritePhase, StagedSegment, StorageProfileIdParseError,
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
