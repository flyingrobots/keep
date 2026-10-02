//! This module owns authenticated reads through an admitted fenced durable view.

mod error;
mod layout_reads;
mod receipt;
mod retained_anchors;
mod snapshot;
mod store;
mod view;

pub use error::{DurableReadError, DurableStoreError};
pub use receipt::{DurableRangeReadReceipt, DurableReconstructionReceipt};
pub use snapshot::DurableSnapshot;
pub use store::{DurableOutcome, DurableStore};
pub use view::DurableView;
