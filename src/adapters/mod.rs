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
mod layout_decode_error;
mod layout_decode_error_display;
mod layout_decode_policy;
mod layout_encode_error;
mod layout_header_decoder;
mod layout_id_binary;
mod layout_id_binary_error;
mod layout_id_text;
mod layout_id_text_error;
mod layout_record;
mod layout_record_decoder;
mod layout_record_encoder;
mod layout_record_format;
mod layout_record_framing;
mod lower_hex;
mod segment_header;
mod segment_header_admission;
mod segment_header_decoder;
mod segment_header_encoding;
mod segment_header_error;
mod segment_header_error_display;
mod storage_profile_id_text;
mod storage_profile_id_text_error;

pub use blob_id_binary_error::BlobIdBinaryParseError;
pub use blob_id_text_error::BlobIdTextParseError;
pub use layout_decode_error::LayoutDecodeError;
pub use layout_decode_policy::LayoutDecodePolicy;
pub use layout_encode_error::LayoutEncodeError;
pub use layout_id_binary_error::LayoutIdBinaryParseError;
pub use layout_id_text_error::LayoutIdTextParseError;
pub use layout_record::CanonicalLayoutRecord;
pub use segment_header::SegmentHeader;
pub use segment_header_error::SegmentHeaderError;
pub use storage_profile_id_text_error::StorageProfileIdParseError;
