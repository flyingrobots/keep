//! Canonical flat-layout identity and validated layout semantics.
//!
//! This module owns logical layout coordinates and laws. Boundary codecs live
//! in `adapters`; physical storage, ingestion, retention, and application
//! policy remain outside this module.

mod admitted;
mod decoded_admission;
mod entry;
mod entry_limit;
mod id;
mod id_mismatch;
mod record_length;
mod validation;
mod validation_error;

pub use admitted::AdmittedLayout;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the reference adapter consumes this domain admission law"
)]
pub(crate) use admitted::check_entry_limit;
pub use entry::LayoutEntry;
pub use entry_limit::{LayoutEntryLimit, LayoutEntryLimitError};
pub use id::LayoutId;
pub use id_mismatch::LayoutIdMismatch;
pub use record_length::LayoutRecordLength;
pub use validation_error::LayoutValidationError;
