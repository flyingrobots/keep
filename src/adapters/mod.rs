//! Canonical boundary codecs.
//!
//! This module owns codecs at Keep's ingress and egress boundaries: decoding
//! raw input into validated domain types, encoding validated domain types
//! into canonical bytes. It does not own identity calculation, storage policy,
//! logical layout policy, physical location, or retention.

mod admitted_segment_record;
mod blob_id_binary;
mod blob_id_binary_error;
mod blob_id_text;
mod blob_id_text_error;
mod checksummed_segment_record;
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
mod segment_digest;
mod segment_header;
mod segment_header_admission;
mod segment_header_decoder;
mod segment_header_encoding;
mod segment_header_error;
mod segment_header_error_display;
mod segment_record_admission;
mod segment_record_admission_error;
mod segment_record_admission_error_display;
mod segment_record_checksum;
mod segment_record_decode_error;
mod segment_record_decode_error_display;
mod segment_record_decoder;
mod segment_record_header;
mod segment_record_header_admission;
mod segment_record_header_decoder;
mod segment_record_header_encoding;
mod segment_record_header_error;
mod segment_record_header_error_display;
mod segment_record_identity;
mod segment_record_identity_admission;
mod segment_record_kind;
mod segment_record_length;
mod segment_record_payload_length;
mod segment_seal;
mod segment_seal_admission;
mod segment_seal_decoder;
mod segment_seal_encoding;
mod segment_seal_error;
mod segment_seal_error_display;
mod segment_seal_hash;
mod storage_profile_id_text;
mod storage_profile_id_text_error;

pub use admitted_segment_record::AdmittedSegmentRecord;
pub use blob_id_binary_error::BlobIdBinaryParseError;
pub use blob_id_text_error::BlobIdTextParseError;
pub use checksummed_segment_record::ChecksummedSegmentRecord;
pub use layout_decode_error::LayoutDecodeError;
pub use layout_decode_policy::LayoutDecodePolicy;
pub use layout_encode_error::LayoutEncodeError;
pub use layout_id_binary_error::LayoutIdBinaryParseError;
pub use layout_id_text_error::LayoutIdTextParseError;
pub use layout_record::CanonicalLayoutRecord;
pub use segment_digest::SegmentDigest;
pub use segment_header::SegmentHeader;
pub use segment_header_error::SegmentHeaderError;
pub use segment_record_admission_error::SegmentRecordAdmissionError;
pub use segment_record_checksum::SegmentRecordChecksum;
pub use segment_record_decode_error::SegmentRecordDecodeError;
pub use segment_record_header::SegmentRecordHeader;
pub use segment_record_header_error::SegmentRecordHeaderError;
pub use segment_record_identity::SegmentRecordIdentity;
pub use segment_record_length::SegmentRecordLength;
pub use segment_record_payload_length::SegmentRecordPayloadLength;
pub use segment_seal::SegmentSeal;
pub use segment_seal_error::SegmentSealError;
pub use storage_profile_id_text_error::StorageProfileIdParseError;
