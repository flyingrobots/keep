//! Exact logical byte identity.
//!
//! This module owns `BlobId` calculation and canonical identity codecs. It does
//! not own storage, layout, representation, location, or retention.

mod hasher;
mod id;
mod id_binary;
mod id_binary_error;
mod id_text;
mod id_text_error;
mod length;

pub use hasher::{BlobHashError, BlobHasher, BlobReadError};
pub use id::BlobId;
pub use id_binary_error::BlobIdBinaryParseError;
pub use id_text_error::BlobIdTextParseError;
pub use length::BlobLength;
