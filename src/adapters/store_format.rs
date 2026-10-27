//! This module owns the canonical `FORMAT` marker and the format-definition digest.
//!
//! The marker is the store's only format-identity record: its magic, version,
//! and definition digest name the format contract, not the software release
//! that wrote a store.

mod admitted_format_marker;
mod canonical_format_marker;
mod format_definition_digest;
mod format_marker_decode_error;
mod format_marker_decode_error_display;
mod format_marker_decoder;
mod format_marker_digest;
mod format_marker_encoder;

pub use admitted_format_marker::AdmittedStoreFormatMarker;
pub use canonical_format_marker::CanonicalStoreFormatMarker;
pub use format_definition_digest::StoreFormatDefinitionDigest;
pub use format_marker_decode_error::StoreFormatMarkerDecodeError;
pub(in crate::adapters) use format_marker_decoder::ENCODED_LENGTH as FORMAT_MARKER_LENGTH;
pub use format_marker_digest::StoreFormatMarkerDigest;
