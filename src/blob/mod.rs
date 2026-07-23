//! Exact logical byte identity.
//!
//! This module owns `BlobId` calculation. It does not own canonical identity
//! codecs, storage, layout, representation, location, or retention.

mod hasher;
mod id;
mod length;

pub use hasher::{BlobHashError, BlobHasher, BlobReadError};
pub use id::BlobId;
pub use length::BlobLength;
