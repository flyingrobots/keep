#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte and physical chunk identity,
//! deterministic streaming chunk detection, canonical flat-layout identity
//! and codecs, and a capacity-bounded non-durable reference CAS. Durable
//! physical storage, retention, and recovery APIs remain intentionally absent
//! until their contracts have executable specifications.

mod adapters;
mod blob;
mod chunk;
mod layout;
mod profile;
mod reference;

pub use adapters::{
    AdmittedSegmentRecord, BlobIdBinaryParseError, BlobIdTextParseError, CanonicalLayoutRecord,
    ChecksummedSegmentRecord, LayoutDecodeError, LayoutDecodePolicy, LayoutEncodeError,
    LayoutIdBinaryParseError, LayoutIdTextParseError, SegmentDigest, SegmentHeader,
    SegmentHeaderError, SegmentRecordAdmissionError, SegmentRecordChecksum,
    SegmentRecordDecodeError, SegmentRecordHeader, SegmentRecordHeaderError, SegmentRecordIdentity,
    SegmentRecordLength, SegmentRecordPayloadLength, SegmentSeal, SegmentSealError,
    StorageProfileIdParseError,
};
pub use blob::{
    BlobHashError, BlobHasher, BlobId, BlobLength, BlobReadError, ByteLength, ByteOffset,
    ByteRange, ByteRangeError,
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
