#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep currently exposes exact logical byte identity. Storage, retention,
//! durability, and recovery APIs remain intentionally absent until their
//! contracts have executable specifications.

mod adapters;
mod blob;

pub use adapters::{BlobIdBinaryParseError, BlobIdTextParseError};
pub use blob::{BlobHashError, BlobHasher, BlobId, BlobLength, BlobReadError};
