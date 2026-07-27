//! Registered deterministic storage-profile identity and admission.
//!
//! This module owns immutable profile coordinates and the closed set of
//! profiles implemented by this Keep version. It does not own layout codecs,
//! profile selection policy, storage, or application metadata.

mod admission_error;
mod id;
mod registered;

pub use admission_error::StorageProfileAdmissionError;
pub use id::StorageProfileId;
pub use registered::RegisteredStorageProfile;
