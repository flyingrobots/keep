//! This module owns classification of typed publication observation failures.

use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use crate::adapters::verification_admission;
use crate::{
    PublicationHeadDecodeError, RetentionCurrentStateRefusal as Current,
    RetentionManifestDecodeError, VerificationRefusal, VerificationSubject,
};
use std::io;

pub(super) fn refusal(error: &io::Error) -> Option<VerificationRefusal> {
    let source = error.get_ref()?;
    if source
        .downcast_ref::<PublicationHeadDecodeError>()
        .is_some()
    {
        return Some(verification_admission::structural(
            VerificationSubject::PublishedCatalog,
        ));
    }
    if let Some(ExactRecordError::Refused(refusal)) = source.downcast_ref::<ExactRecordError>() {
        return exact_record_refusal(*refusal);
    }
    current_refusal(source.downcast_ref::<Current>()?)
}

const fn exact_record_refusal(refusal: ExactRecordRefusal) -> Option<VerificationRefusal> {
    match refusal {
        ExactRecordRefusal::LengthOverflow => None,
        ExactRecordRefusal::KindOrLength
        | ExactRecordRefusal::KindLengthOrIdentity
        | ExactRecordRefusal::Bytes
        | ExactRecordRefusal::TrailingBytes
        | ExactRecordRefusal::RemainedVisible => Some(verification_admission::structural(
            VerificationSubject::PublishedView,
        )),
    }
}

const fn current_refusal(current: &Current) -> Option<VerificationRefusal> {
    match current {
        Current::ManifestAbsent | Current::CatalogAbsent => Some(VerificationRefusal::Missing {
            subject: VerificationSubject::PublishedView,
        }),
        Current::ManifestRefused {
            source: RetentionManifestDecodeError::Allocation { .. },
        }
        | Current::GcIntentRetained
        | Current::ClosureMemberRefused { .. }
        | Current::ClosureReverificationRefused { .. }
        | Current::ClosureDigestChanged
        | Current::RetainedStage
        | Current::HeadAbsentWithArtifacts
        | Current::ExpectedCurrentOverAbsentHead
        | Current::NonInitialOverAbsentHead
        | Current::PreparedHeadRefused { .. }
        | Current::CatalogDisagreed { .. }
        | Current::LivenessExhausted
        | Current::StaleCommittedRetry
        | Current::Superseded { .. }
        | Current::CommittedSelectionMissing
        | Current::CommittedSelectionMismatch
        | Current::NamespaceRead { .. }
        | Current::CommittedNamespaceUnavailable
        | Current::CommittedRootAbsent
        | Current::CommittedRootChanged
        | Current::PredecessorMismatch
        | Current::PredecessorRootAbsent
        | Current::PredecessorRootRefused { .. }
        | Current::PredecessorRootChanged
        | Current::UnknownRetentionEntry
        | Current::NonNamespaceEntry
        | Current::NoncanonicalPoolEntry { .. }
        | Current::NamespaceCapacity
        | Current::NamespaceExpectationViolated
        | Current::AttemptNamespaceDisagreed
        | Current::CommittedRetryOverAbsentHead
        | Current::ProtocolDirectoryReplaced
        | Current::RecoveryObservationRefused { .. }
        | Current::RecoveryRefused { .. }
        | Current::RecoveryStepRefused { .. }
        | Current::RecordLengthOverflow => None,
        Current::HeadRefused { .. }
        | Current::ManifestRefused { .. }
        | Current::ManifestDisagreed
        | Current::HeadPredecessorDisagreed
        | Current::RecordKindOrLength
        | Current::RecordTrailingBytes
        | Current::CatalogHeadRefused { .. }
        | Current::CatalogRefused { .. }
        | Current::CatalogChanged => Some(verification_admission::structural(
            VerificationSubject::PublishedView,
        )),
    }
}
