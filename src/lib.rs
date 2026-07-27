#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte and physical chunk identity,
//! deterministic streaming chunk detection, and canonical flat-layout
//! identity, admission, encoding, and bounded decoding. Physical storage,
//! retention, durability, and recovery APIs remain intentionally absent until
//! their contracts have executable specifications.

mod adapters;
mod blob;
mod chunk;
mod layout;
mod profile;

pub use adapters::{
    BlobIdBinaryParseError, BlobIdTextParseError, CanonicalLayoutRecord, LayoutDecodeError,
    LayoutDecodePolicy, LayoutEncodeError, LayoutIdBinaryParseError, LayoutIdTextParseError,
    StorageProfileIdParseError,
};
pub use blob::{BlobHashError, BlobHasher, BlobId, BlobLength, BlobReadError};
pub use chunk::{
    ChunkHashError, ChunkId, ChunkLength, ChunkOffset, ChunkSpan, ChunkingError, FastCdc,
};
pub use layout::{
    AdmittedLayout, LayoutEntry, LayoutEntryLimit, LayoutEntryLimitError, LayoutId,
    LayoutIdMismatch, LayoutRecordLength, LayoutValidationError,
};
pub use profile::{RegisteredStorageProfile, StorageProfileAdmissionError, StorageProfileId};
