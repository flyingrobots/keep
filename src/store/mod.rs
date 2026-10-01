//! The backend-neutral content-store port.
//!
//! `ContentReads` is what every admitted view answers: `contains_blob`,
//! `reconstruct`, `reconstruct_layout`, `read_range`, and
//! `read_layout_range`, with the reference store's laws. `ContentStaging`
//! and `StagedContent` are the write half: stage a source under
//! count-and-byte limits, then commit the staged content into its store
//! for a receipt. The non-durable `ReferenceStore` implements both halves
//! and stays honest about being non-durable; the durable `DurableSnapshot`
//! implements the read half, and the durable writer lands with the
//! bounded ingestion path.
//!
//! Receipts are distinct types per backend, so a non-durable receipt can
//! never stand where a durable one is required:
//!
//! ```compile_fail
//! fn requires_durable(_receipt: keep::DurableReconstructionReceipt) {}
//! fn hand_over(receipt: keep::ReconstructionReceipt) {
//!     requires_durable(receipt);
//! }
//! ```

mod limits;
#[cfg(test)]
pub(crate) mod port_laws;
mod reads;
#[cfg(test)]
mod reference_port_tests;
mod staging;
mod transfer_source;

pub use limits::{StagedByteLimit, StagingLimits};
pub use reads::ContentReads;
pub use staging::{CommitReceipt, ContentStaging, StagedContent};
#[expect(
    clippy::redundant_pub_crate,
    reason = "adapter sealing is reachable only through this crate-private port re-export"
)]
pub(crate) use transfer_source::sealed::Sealed as SealedTransferSource;
pub use transfer_source::{StreamConsumer, TransferSource, TransferSourceError};
