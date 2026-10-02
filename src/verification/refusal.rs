//! This module owns semantic refusals of verification requests.

use std::error::Error;
use std::fmt;

use super::{VerificationDepth, VerificationSubject};

/// Why a verification request cannot establish its requested evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationRefusal {
    /// The operation cannot establish this depth for the requested subject.
    Unsupported {
        /// Subject the caller asked to verify.
        subject: VerificationSubject,
        /// Exact requested depth; the operation does not silently downgrade it.
        requested: VerificationDepth,
        /// Exact supported set, which need not be a contiguous ordinal range.
        supported: &'static [VerificationDepth],
    },
}

impl fmt::Display for VerificationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { requested, .. } => {
                write!(
                    formatter,
                    "verification depth {requested:?} is unsupported for this subject"
                )
            }
        }
    }
}

impl Error for VerificationRefusal {}
