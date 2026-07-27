//! Canonical identity codecs.
//!
//! This module owns codecs at Keep's ingress and egress boundaries: decoding
//! raw input into validated domain types, encoding validated domain types
//! into canonical bytes. It does not own identity calculation, storage,
//! layout, representation, location, or retention.

mod blob_id_binary;
mod blob_id_binary_error;
mod blob_id_text;
mod blob_id_text_error;
mod layout_id_binary;
mod layout_id_binary_error;
mod layout_id_text;
mod layout_id_text_error;
mod lower_hex;
mod storage_profile_id_text;
mod storage_profile_id_text_error;

pub use blob_id_binary_error::BlobIdBinaryParseError;
pub use blob_id_text_error::BlobIdTextParseError;
pub use layout_id_binary_error::LayoutIdBinaryParseError;
pub use layout_id_text_error::LayoutIdTextParseError;
pub use storage_profile_id_text_error::StorageProfileIdParseError;
