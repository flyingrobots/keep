//! This module owns semantic refusals of verification requests.

use std::error::Error;
use std::fmt;

use super::{VerificationDepth, VerificationObservation, VerificationSubject};

/// Why a verification request cannot establish its requested evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationRefusal {
    /// Required evidence is absent from an admitted immutable view.
    Missing {
        /// Exact absent logical subject.
        subject: VerificationSubject,
    },
    /// Present evidence contradicts the requirement at this boundary.
    Corrupt {
        /// Subject whose evidence failed admission.
        subject: VerificationSubject,
        /// Required coordinate or structural predicate.
        expected: VerificationObservation,
        /// Observed coordinate or failed predicate.
        observed: VerificationObservation,
    },
    /// Conflicting observations prevent selection of one admissible view.
    Ambiguous {
        /// Bounded pair of conflicting observations, not an exhaustive inventory.
        candidates: [VerificationSubject; 2],
    },

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
            Self::Missing { .. } => formatter.write_str("required verification evidence is absent"),
            Self::Corrupt { .. } => {
                formatter.write_str("verification evidence contradicts its contract")
            }
            Self::Ambiguous { .. } => formatter.write_str("verification observations conflict"),

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
