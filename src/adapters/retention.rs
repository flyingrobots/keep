//! This module owns canonical retention record boundary adapters.

mod admitted_root;
mod canonical_root;
mod root_anchor_decoder;
mod root_decode_error;
mod root_decode_error_display;
mod root_decoder;
mod root_encode_error;
mod root_encoder;
mod root_field_decoder;
mod root_header_decoder;
mod root_integrity;
mod root_semantic_header;

pub use admitted_root::AdmittedRetentionRoot;
pub use canonical_root::CanonicalRetentionRoot;
pub use root_decode_error::RetentionRootDecodeError;
pub use root_encode_error::RetentionRootEncodeError;
