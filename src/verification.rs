//! This module owns immutable statements of achieved verification evidence.
//!
//! Evidence is specific to a subject. A depth's ordering is not permission to
//! infer a different subject's properties, publication, or retention authority.

mod depth;
mod evidence;
mod layout_details;
pub use layout_details::ReferenceVerificationDetails;
mod observation;
mod refusal;
mod report;
mod subject;

pub use depth::VerificationDepth;
pub use observation::VerificationObservation;
pub use refusal::VerificationRefusal;
pub use report::{VerificationReport, VerifiedSubject};
pub use subject::VerificationSubject;

pub use evidence::{CorruptionEvidence, MissingEvidence};
