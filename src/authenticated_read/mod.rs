//! This module owns authentication and exact emission of admitted logical reads.
//!
//! Adapters supply immutable chunk bytes and admitted layouts. This core owns
//! verification policy, output accounting and proof receipts; it never decodes
//! records, opens storage, resolves publication, or imports adapter errors.

#![expect(
    clippy::redundant_pub_crate,
    reason = "semantic ports and failures remain crate-private if the module is later exposed"
)]
#![expect(
    clippy::result_large_err,
    reason = "retain exact identity coordinates and original causes without heap allocation"
)]

mod chunk_reader;
mod chunk_verification;
mod output_write;
mod profile_verification;
mod range_read_execution;
mod range_read_failure;
mod range_read_receipt;
mod reconstruction;
mod reconstruction_failure;
mod reconstruction_receipt;

pub(crate) use chunk_reader::ChunkReader;
pub(crate) use chunk_verification::{ChunkSource, ChunkVerificationError, verified_chunk};
pub(crate) use output_write::OutputWriteError;
pub(crate) use range_read_execution::read_admitted;
pub(crate) use range_read_failure::RangeReadFailure;
pub use range_read_receipt::RangeReadReceipt;
pub(crate) use reconstruction::reconstruct_admitted;
pub(crate) use reconstruction_failure::ReconstructionFailure;
pub use reconstruction_receipt::ReconstructionReceipt;
