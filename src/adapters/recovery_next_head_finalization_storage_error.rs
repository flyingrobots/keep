//! This module owns semantic storage failures during next-head finalization.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    CatalogPublicationExpectation, CatalogRestartError, FilesystemRecoveryStageError,
    RecoveryNextHeadFinalizationTarget, RecoveryStageEvidence,
};

/// Why storage could not continue one exact next-head finalization request.
#[derive(Debug)]
pub enum RecoveryNextHeadFinalizationStorageError {
    /// The canonical `head.next` could not be observed exactly.
    Stage {
        /// Exact no-follow stage observation failure.
        source: Box<FilesystemRecoveryStageError>,
    },
    /// The canonical `head.next` resolves to different evidence.
    EvidenceMismatch {
        /// Evidence bound into the explicit finalization request.
        expected: RecoveryStageEvidence,
        /// Evidence observed immediately before the refused transition.
        observed: RecoveryStageEvidence,
    },
    /// Durable `HEAD` is neither the expected current head nor the candidate.
    CurrentMismatch {
        /// Current-state coordinate bound into the request.
        expected: CatalogPublicationExpectation,
        /// Different valid durable head, or absence.
        observed: Option<RecoveryNextHeadFinalizationTarget>,
    },
    /// The complete candidate view resolves to different coordinates.
    CandidateMismatch {
        /// Candidate coordinate bound into the request.
        expected: RecoveryNextHeadFinalizationTarget,
        /// Different complete candidate coordinate.
        observed: RecoveryNextHeadFinalizationTarget,
    },
    /// The candidate is absent while durable `HEAD` still matches the expectation.
    MissingCandidate {
        /// Exact missing `head.next` evidence bound into the request.
        expected: RecoveryStageEvidence,
    },
    /// Durable `HEAD` is final but the fixed candidate name reappeared.
    UnexpectedCandidate {
        /// Evidence from the request whose completed retry requires absence.
        expected: RecoveryStageEvidence,
    },
    /// The complete candidate snapshot could not be reconstructed.
    CandidateView {
        /// Exact bounded restart-loading failure.
        source: Box<CatalogRestartError>,
    },
    /// Durable current `HEAD` could not be reconstructed.
    CurrentView {
        /// Exact bounded restart-loading failure.
        source: Box<CatalogRestartError>,
    },
    /// The storage boundary failed while observing or mutating.
    Storage {
        /// Exact underlying storage failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryNextHeadFinalizationStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stage { source } => {
                write!(formatter, "recovery next-head observation failed: {source}")
            }
            Self::EvidenceMismatch { expected, observed } => write!(
                formatter,
                "{} finalization evidence changed from length {} to length {}",
                expected.stage(),
                expected.length().get(),
                observed.length().get()
            ),
            Self::CurrentMismatch { expected, observed } => write!(
                formatter,
                "durable head does not match expected generation {:?} digest {:?}; observed {observed:?}",
                expected.current_generation(),
                expected.current_catalog_digest()
            ),
            Self::CandidateMismatch { expected, observed } => write!(
                formatter,
                "candidate generation {} length {} digest {:?} does not match generation {} length {} digest {:?}",
                expected.generation().get(),
                expected.length().get(),
                expected.digest(),
                observed.generation().get(),
                observed.length().get(),
                observed.digest()
            ),
            Self::MissingCandidate { expected } => write!(
                formatter,
                "{} candidate is absent for evidence length {}",
                expected.stage(),
                expected.length().get()
            ),
            Self::UnexpectedCandidate { expected } => write!(
                formatter,
                "{} candidate reappeared after finalization for evidence length {}",
                expected.stage(),
                expected.length().get()
            ),
            Self::CandidateView { source } => {
                write!(formatter, "recovery candidate view is invalid: {source}")
            }
            Self::CurrentView { source } => {
                write!(formatter, "durable current view is invalid: {source}")
            }
            Self::Storage { source } => {
                write!(
                    formatter,
                    "recovery next-head finalization failed: {source}"
                )
            }
        }
    }
}

impl Error for RecoveryNextHeadFinalizationStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            Self::Stage { source } => Some(source.as_ref()),
            Self::CandidateView { source } | Self::CurrentView { source } => Some(source.as_ref()),
            Self::EvidenceMismatch { .. }
            | Self::CurrentMismatch { .. }
            | Self::CandidateMismatch { .. }
            | Self::MissingCandidate { .. }
            | Self::UnexpectedCandidate { .. } => None,
        }
    }
}
