//! This module owns lossless semantic classification of verification failures.

use super::verification_failure_class::{FailureClass, layout_class};
use super::{VerificationError, VerificationSource};
use crate::{
    LayoutDecodeError, RetentionClosureVerificationError as Closure, SegmentRecordIdentity,
    VerificationObservation as Observation, VerificationRefusal, VerificationSubject,
};

pub(super) fn layout(subject: VerificationSubject, source: LayoutDecodeError) -> VerificationError {
    let operational = matches!(layout_class(&source), FailureClass::Operational);
    let source = Box::new(VerificationSource::Layout(source));
    if operational {
        return VerificationError::Operational { source };
    }
    VerificationError::Refused {
        refusal: structural(subject),
        source: Some(source),
    }
}

pub(super) fn closure(subject: VerificationSubject, source: Closure) -> VerificationError {
    let refusal = match &source {
        Closure::MissingMember { identity } => VerificationRefusal::Missing {
            subject: member(*identity),
        },
        Closure::AnchorTargetMismatch {
            expected, observed, ..
        }
        | Closure::BlobIdentityMismatch {
            expected, observed, ..
        } => VerificationRefusal::Corrupt {
            subject,
            expected: Observation::Blob(*expected),
            observed: Observation::Blob(*observed),
        },
        Closure::ProfileBoundaryMismatch {
            expected, observed, ..
        } => VerificationRefusal::Corrupt {
            subject,
            expected: Observation::ProfileBoundary(*expected),
            observed: Observation::ProfileBoundary(*observed),
        },
        Closure::LayoutDecode { .. } => structural(subject),
        Closure::CounterOverflow { .. }
        | Closure::LimitExceeded { .. }
        | Closure::ProfileVerifierUnavailable { .. }
        | Closure::ProfileChunking { .. }
        | Closure::BlobHash { .. } => {
            return VerificationError::Operational {
                source: Box::new(VerificationSource::Closure(source)),
            };
        }
    };
    if let Closure::LayoutDecode { source: nested, .. } = &source
        && matches!(layout_class(nested), FailureClass::Operational)
    {
        return VerificationError::Operational {
            source: Box::new(VerificationSource::Closure(source)),
        };
    }
    VerificationError::Refused {
        refusal,
        source: Some(Box::new(VerificationSource::Closure(source))),
    }
}

pub(super) const fn structural(subject: VerificationSubject) -> VerificationRefusal {
    VerificationRefusal::Corrupt {
        subject,
        expected: Observation::Canonical,
        observed: Observation::Refused,
    }
}

const fn member(identity: SegmentRecordIdentity) -> VerificationSubject {
    match identity {
        SegmentRecordIdentity::Chunk(identity) => VerificationSubject::Chunk { identity },
        SegmentRecordIdentity::Layout(identity) => VerificationSubject::Layout { identity },
    }
}
