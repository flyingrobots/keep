//! This module owns the public read error boundary shared by storage adapters.
//!
//! It combines codec/lookup refusals with lossless translation of semantic
//! core failures. Authentication policy remains in the inward read core.

mod profile_error_mapping;
mod range_failure_mapping;
mod range_read_error;
mod range_read_error_display;
mod range_read_error_mapping;
mod reconstruction_error;
mod reconstruction_error_display;
mod reconstruction_error_mapping;
mod reconstruction_failure_mapping;

pub use range_read_error::RangeReadError;
pub use reconstruction_error::ReconstructionError;
