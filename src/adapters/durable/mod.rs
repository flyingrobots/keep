//! Durable authenticated reads over one fenced version-two snapshot.
//!
//! `DurableStore` names a migrated store; `DurableSnapshot` pins one
//! consistent view under the shared reader fence and answers
//! `reconstruct`, `reconstruct_layout`, `read_range`, and
//! `read_layout_range` with exactly the reference store's laws, resolving
//! layouts and chunks through the fenced catalog and blobs through the
//! retained anchors. Every receipt names the view it was established
//! against. While a snapshot lives, collection cannot retire what it reads.
//!
//! `DurableWriter` is the write half: one bounded pass stages a source into
//! a segment stage, reusing every chunk the pinned catalog already holds
//! after exact byte comparison, and commit publishes the segment and a
//! catalog successor through the version-one protocol.

mod error;
mod ingestion_error;
mod ingestion_receipt;
mod port;
#[cfg(test)]
mod port_tests;
mod receipt;
mod recovery;
mod snapshot;
mod staged;
mod store;
#[cfg(test)]
mod test_fixture;
#[cfg(test)]
mod tests;
mod transfer_source;
#[cfg(test)]
mod transfer_tests;
mod view;
mod writer;
mod writer_sink;
#[cfg(test)]
mod writer_tests;

pub use error::{DurableReadError, DurableStoreError};
pub use ingestion_error::DurableIngestionError;
pub use ingestion_receipt::{DurableIngestionReceipt, IngestionAccounting};
pub use receipt::{DurableRangeReadReceipt, DurableReconstructionReceipt};
pub use recovery::recover_durable_ingestion;
pub use snapshot::DurableSnapshot;
pub use staged::DurableStagedBlob;
pub use store::{DurableOutcome, DurableStore};
pub use view::DurableView;
pub use writer::DurableWriter;
