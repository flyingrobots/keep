//! Semantic retention coordinates and reconstruction anchors.
//!
//! This module owns validated namespace bytes, namespace identity,
//! generation coordinates, and logical reconstruction anchors. It does not own
//! record encoding, filesystem layout, publication, recovery, or garbage
//! collection.

mod anchor;
mod liveness_generation;
mod liveness_generation_error;
mod namespace;
mod namespace_digest;
mod namespace_error;
mod root_generation;
mod root_generation_error;

pub use anchor::RetentionAnchor;
pub use liveness_generation::LivenessGeneration;
pub use liveness_generation_error::LivenessGenerationError;
pub use namespace::RetentionNamespace;
pub use namespace_digest::RetentionNamespaceDigest;
pub use namespace_error::RetentionNamespaceError;
pub use root_generation::RootGeneration;
pub use root_generation_error::RootGenerationError;
