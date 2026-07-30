//! This module owns canonical retention record boundary adapters.

mod canonical_root;
mod root_encode_error;
mod root_encoder;

pub use canonical_root::CanonicalRetentionRoot;
pub use root_encode_error::RetentionRootEncodeError;
