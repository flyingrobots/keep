//! Explicit verification depths, reports, and refusals.
//!
//! This module owns the vocabulary every Keep verification operation uses to
//! say exactly what it established and exactly why it stopped. It does not
//! own any store, any read path, or any repair: a report is a statement, a
//! refusal is evidence, and neither mutates physical state.

mod depth;
mod error;
mod error_display;
mod evidence;
mod refusal;
mod report;
mod subject;

pub use depth::VerificationDepth;
pub use error::{VerificationError, VerificationFailure};
pub use evidence::{CorruptionEvidence, MissingEvidence};
pub use refusal::VerificationRefusal;
pub use report::VerificationReport;
pub use subject::VerificationSubject;
