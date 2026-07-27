#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte and physical chunk identity plus
//! deterministic streaming chunk detection. Layout, storage, retention,
//! durability, and recovery APIs remain intentionally absent until their
//! contracts have executable specifications.

mod adapters;
mod blob;
mod chunk;
mod layout;
mod profile;

pub use adapters::{
    BlobIdBinaryParseError, BlobIdTextParseError, LayoutIdBinaryParseError, LayoutIdTextParseError,
    StorageProfileIdParseError,
};
pub use blob::{BlobHashError, BlobHasher, BlobId, BlobLength, BlobReadError};
pub use chunk::{
    ChunkHashError, ChunkId, ChunkLength, ChunkOffset, ChunkSpan, ChunkingError, FastCdc,
};
pub use layout::{LayoutId, LayoutIdMismatch, LayoutRecordLength};
pub use profile::{RegisteredStorageProfile, StorageProfileAdmissionError, StorageProfileId};
