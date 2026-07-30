//! Semantic retention coordinates and reconstruction anchors.
//!
//! This module owns validated namespace bytes, namespace identity,
//! generation coordinates, and logical reconstruction anchors. It does not own
//! record encoding, filesystem layout, publication, recovery, or garbage
//! collection.

mod anchor;
mod closure_counter;
mod closure_digest;
mod closure_limit;
mod closure_limit_error;
mod closure_limits;
mod closure_usage;
mod generation_expectation;
mod head;
mod head_error;
mod liveness_generation;
mod liveness_generation_error;
mod manifest;
mod manifest_digest;
mod manifest_entry;
mod manifest_error;
mod manifest_length;
mod manifest_length_error;
mod namespace;
mod namespace_digest;
mod namespace_error;
mod policy;
mod profile;
mod profile_admission_error;
mod root;
mod root_digest;
mod root_error;
mod root_generation;
mod root_generation_error;

pub use anchor::RetentionAnchor;
pub use closure_counter::RetentionClosureCounter;
pub use closure_digest::RetentionClosureDigest;
pub use closure_limit::RetentionClosureLimit;
pub use closure_limit_error::RetentionClosureLimitError;
pub use closure_limits::RetentionClosureLimits;
pub use closure_usage::RetentionClosureUsage;
pub use generation_expectation::RetentionGenerationExpectation;
pub use head::RetentionHead;
pub use head_error::RetentionHeadError;
pub use liveness_generation::LivenessGeneration;
pub use liveness_generation_error::LivenessGenerationError;
pub use manifest::RetentionManifest;
pub use manifest_digest::RetentionManifestDigest;
pub use manifest_entry::RetentionManifestEntry;
pub use manifest_error::RetentionManifestError;
pub use manifest_length::RetentionManifestLength;
pub use manifest_length_error::RetentionManifestLengthError;
pub use namespace::RetentionNamespace;
pub use namespace_digest::RetentionNamespaceDigest;
pub use namespace_error::RetentionNamespaceError;
pub use policy::RetentionPolicy;
pub use profile::RegisteredRetentionProfile;
pub use profile_admission_error::RetentionProfileAdmissionError;
pub use root::RetentionRoot;
pub use root_digest::RetentionRootDigest;
pub use root_error::RetentionRootError;
pub use root_generation::RootGeneration;
pub use root_generation_error::RootGenerationError;
