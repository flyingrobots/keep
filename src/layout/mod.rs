//! Canonical flat-layout identity and validated layout semantics.
//!
//! This module owns logical layout coordinates and laws. Boundary codecs live
//! in `adapters`; physical storage, ingestion, retention, and application
//! policy remain outside this module.

mod id;
mod id_mismatch;
mod record_length;

pub use id::LayoutId;
pub use id_mismatch::LayoutIdMismatch;
pub use record_length::LayoutRecordLength;
