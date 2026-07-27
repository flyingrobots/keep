//! Capacity-bounded, non-durable reference storage adapter.
//!
//! This adapter deliberately materializes stored chunk bytes in memory. Its
//! capacity is explicit, staged work is invisible until commit, and no API
//! makes a retention, crash-recovery, or durability claim.

mod capacity;
mod chunk_staging;
mod ingestion;
mod ingestion_error;
mod publish_error;
mod published_blob;
mod staged_blob;
mod store;

pub use capacity::ReferenceStoreCapacity;
pub use ingestion_error::{IngestionAllocation, IngestionError};
pub use publish_error::PublishError;
pub use published_blob::PublishedBlob;
pub use staged_blob::StagedBlob;
pub use store::ReferenceStore;
