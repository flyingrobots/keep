//! Canonical identity codecs.
//!
//! This module owns the canonical binary and text wire representations for
//! domain identities: parsing raw input into validated domain types, and
//! encoding validated domain types back into their canonical bytes. It does
//! not own identity calculation, storage, layout, representation, location,
//! or retention.

mod blob_id_binary;
mod blob_id_binary_error;
mod blob_id_text;
mod blob_id_text_error;

pub use blob_id_binary_error::BlobIdBinaryParseError;
pub use blob_id_text_error::BlobIdTextParseError;
