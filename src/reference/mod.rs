//! Capacity-bounded, non-durable reference storage adapter.
//!
//! This adapter deliberately materializes stored chunk bytes in memory. Its
//! capacity is explicit, staged work is invisible until commit, and no API
//! makes a retention, crash-recovery, or durability claim.

mod capacity;
mod chunk_staging;
mod ingestion;
mod ingestion_error;
mod profile_boundary;
mod profile_verification;
mod publish_error;
mod published_blob;
mod reconstruction;
mod reconstruction_error;
mod reconstruction_error_display;
mod reconstruction_receipt;
mod staged_blob;
mod store;

pub use capacity::ReferenceStoreCapacity;
pub use ingestion_error::{IngestionAllocation, IngestionError};
pub use profile_boundary::ProfileBoundary;
pub use publish_error::PublishError;
pub use published_blob::PublishedBlob;
pub use reconstruction_error::ReconstructionError;
pub use reconstruction_receipt::ReconstructionReceipt;
pub use staged_blob::StagedBlob;
pub use store::ReferenceStore;
