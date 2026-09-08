//! Registered deterministic storage-profile identity and admission.
//!
//! This module owns immutable profile coordinates and the closed set of
//! profiles implemented by this Keep version. It does not own layout codecs,
//! profile selection policy, storage, or application metadata.

mod admission_error;
mod boundary;
mod id;
mod registered;
mod verification;
mod verification_error;

pub use admission_error::StorageProfileAdmissionError;
pub use boundary::ProfileBoundary;
pub use id::StorageProfileId;
pub use registered::RegisteredStorageProfile;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling adapters share profile replay without exposing it publicly"
)]
pub(crate) use verification::StorageProfileVerifier;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling adapters map the same domain replay failures"
)]
pub(crate) use verification_error::StorageProfileVerificationError;
