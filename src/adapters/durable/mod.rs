//! Durable authenticated reads over one fenced version-two snapshot.
//!
//! `DurableStore` names a migrated store; `DurableSnapshot` pins one
//! consistent view under the shared reader fence and answers
//! `reconstruct`, `reconstruct_layout`, `read_range`, and
//! `read_layout_range` with exactly the reference store's laws, resolving
//! layouts and chunks through the fenced catalog and blobs through the
//! retained anchors. Every receipt names the view it was established
//! against. While a snapshot lives, collection cannot retire what it reads.

mod error;
mod port;
#[cfg(test)]
mod port_tests;
mod receipt;
mod snapshot;
mod store;
#[cfg(test)]
mod test_fixture;
#[cfg(test)]
mod tests;
mod view;

pub use error::{DurableReadError, DurableStoreError};
pub use receipt::{DurableRangeReadReceipt, DurableReconstructionReceipt};
pub use snapshot::DurableSnapshot;
pub use store::{DurableOutcome, DurableStore};
pub use view::DurableView;
