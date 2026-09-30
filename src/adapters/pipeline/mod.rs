//! The bounded streaming write-through pipeline.
//!
//! `transfer_layout`, `transfer_blob`, `transfer_range`, and
//! `transfer_layout_range` move authenticated bytes from any `ContentReads`
//! view into a `TransferSink` as verified segments: each segment is the
//! read core's own borrowed slice of an immutable chunk, handed over
//! without a copy, applied exactly once in order, and acknowledged every
//! `TransferWindow` segments. A `CancellationSignal` is consulted before
//! every segment, and cancellation never upgrades partial output into a
//! receipt.
//!
//! `copy_layout` moves a blob between stores without buffering it: a
//! `TransferSource` streams the layout's chunks through a pull reader that
//! authenticates each chunk as it is served, and the destination's own
//! staging recomputes and checks the complete identity.

mod cancellation;
mod copy;
mod error;
mod receipt;
mod sink;
mod source;
#[cfg(test)]
mod tests;
mod transfer;
mod window;

pub use cancellation::{CancellationFlag, CancellationSignal, NeverCancelled};
pub use copy::{CopyError, CopyReceipt, copy_layout};
pub use error::TransferError;
pub use receipt::TransferReceipt;
pub use sink::{TransferSegment, TransferSink, WriteSink, WriteSinkError};
pub use source::{StreamConsumer, TransferSource, TransferSourceError};
pub use transfer::{
    TransferBounds, transfer_blob, transfer_layout, transfer_layout_range, transfer_range,
};
pub use window::TransferWindow;
