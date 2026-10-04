//! This module owns typed decoder-to-verification error admission.

use super::{VerificationError, VerificationSource, verification_admission};
use crate::{LayoutDecodeError, RetentionRootDecodeError, VerificationSubject};

impl From<LayoutDecodeError> for VerificationError {
    /// Classifies a raw layout failure, preserving its original typed cause.
    /// Allocates only a boxed error; performs no I/O or verification upgrade.
    fn from(source: LayoutDecodeError) -> Self {
        verification_admission::layout(VerificationSubject::LayoutInput, source)
    }
}

impl From<RetentionRootDecodeError> for VerificationError {
    /// Classifies a raw root failure without asserting an admitted root identity.
    /// Allocation failure is operational; contradictory bytes are corruption.
    /// Allocates only a boxed error and performs no I/O.
    fn from(source: RetentionRootDecodeError) -> Self {
        if matches!(source, RetentionRootDecodeError::Allocation { .. }) {
            return Self::Operational {
                source: Box::new(VerificationSource::Root(source)),
            };
        }
        Self::Refused {
            refusal: verification_admission::structural(VerificationSubject::RetentionRootInput),
            source: Some(Box::new(VerificationSource::Root(source))),
        }
    }
}
