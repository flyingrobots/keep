//! Exact logical byte identity.
//!
//! This module owns `BlobId` calculation. It does not own canonical identity
//! codecs, storage, layout, representation, location, or retention.

mod byte_length;
mod byte_offset;
mod byte_range;
mod hasher;
mod id;
mod length;

pub use byte_length::ByteLength;
pub use byte_offset::ByteOffset;
pub use byte_range::{ByteRange, ByteRangeError};
pub use hasher::{BlobHashError, BlobHasher, BlobReadError};
pub use id::BlobId;
pub use length::BlobLength;
