//! Capacity-bounded, non-durable reference storage adapter.
//!
//! This adapter deliberately materializes stored chunk bytes in memory. Its
//! capacity is explicit, staged work is invisible until commit, and no API
//! makes a retention, crash-recovery, or durability claim.

mod capacity;
mod chunk_source;
mod chunk_staging;
mod ingestion;
mod ingestion_error;
mod port;
mod transfer_source;
mod verification;
mod verification_context;
mod verification_outcome;
mod verification_source;
pub use verification_context::{ReferenceVerificationContext, ReferenceVerificationEvidence};
pub use verification_source::ReferenceVerificationSource;
mod publish_error;
mod published_blob;
mod range_read;
mod reconstruction;
mod staged_blob;
mod store;

pub use capacity::ReferenceStoreCapacity;
#[expect(
    clippy::redundant_pub_crate,
    reason = "staging capabilities stay crate-private across adapters"
)]
pub(crate) use ingestion::{ChunkSink, ingest_stream};
pub use ingestion_error::{IngestionAllocation, IngestionError};
pub use port::ReferenceStagedContent;
pub use publish_error::PublishError;
pub use published_blob::PublishedBlob;
pub use staged_blob::StagedBlob;
pub use store::ReferenceStore;

use crate::{RangeReadError, RangeReadReceipt, ReconstructionError, ReconstructionReceipt};
