//! Capacity-bounded, non-durable reference storage adapter.
//!
//! This adapter deliberately materializes stored chunk bytes in memory. Its
//! capacity is explicit, staged work is invisible until commit, and no API
//! makes a retention, crash-recovery, or durability claim.

mod capacity;
mod chunk_staging;
mod chunk_verification;
mod ingestion;
mod ingestion_error;
mod output_write;
mod port;
mod profile_verification;
mod publish_error;
mod published_blob;
mod range_read;
mod range_read_error;
mod range_read_error_display;
mod range_read_error_mapping;
mod range_read_execution;
mod range_read_receipt;
mod reconstruction;
mod reconstruction_error;
mod reconstruction_error_display;
mod reconstruction_receipt;
mod staged_blob;
mod store;
mod verification;

pub use crate::profile::ProfileBoundary;
pub use capacity::ReferenceStoreCapacity;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the durable adapter reaches the shared read cores through this crate-private surface"
)]
pub(crate) use chunk_verification::ChunkSource;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the durable adapter reaches the streaming core through this crate-private surface"
)]
pub(crate) use ingestion::{ChunkSink, ingest_stream};
pub use ingestion_error::{IngestionAllocation, IngestionError};
pub use port::ReferenceStagedContent;
pub use publish_error::PublishError;
pub use published_blob::PublishedBlob;
pub use range_read_error::RangeReadError;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the durable adapter reaches the shared read cores through this crate-private surface"
)]
pub(crate) use range_read_execution::read_admitted;
pub use range_read_receipt::RangeReadReceipt;
#[expect(
    clippy::redundant_pub_crate,
    reason = "the durable adapter reaches the shared read cores through this crate-private surface"
)]
pub(crate) use reconstruction::reconstruct_admitted;
pub use reconstruction_error::ReconstructionError;
pub use reconstruction_receipt::ReconstructionReceipt;
pub use staged_blob::StagedBlob;
pub use store::ReferenceStore;
