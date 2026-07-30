//! This boundary module owns retention publication preparation failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use super::RetentionManifestEncodeError;
use crate::{
    LivenessGenerationError, RetentionHeadError, RetentionManifestError,
    RetentionManifestLengthError, RetentionNamespaceDigest, RetentionRootDigest, RootGeneration,
    RootGenerationError,
};

/// Failure to bind preflight evidence into canonical global artifacts.
#[derive(Debug)]
pub enum RetentionPublicationPreparationError {
    /// Exact retry had no current global manifest to select its root.
    CurrentManifestRequired {
        /// Candidate namespace that must be selected.
        namespace: RetentionNamespaceDigest,
    },
    /// Exact retry was absent from the current global manifest.
    CurrentManifestEntryMissing {
        /// Candidate namespace absent from the manifest.
        namespace: RetentionNamespaceDigest,
        /// Candidate root generation.
        generation: RootGeneration,
        /// Candidate root digest.
        digest: RetentionRootDigest,
    },
    /// Current manifest entry and candidate did not form a successor.
    ManifestSuccessorMismatch {
        /// Candidate namespace.
        namespace: RetentionNamespaceDigest,
        /// Generation selected by the current manifest.
        current_generation: RootGeneration,
        /// Root digest selected by the current manifest.
        current_digest: RetentionRootDigest,
        /// Candidate root generation.
        candidate_generation: RootGeneration,
        /// Candidate-declared predecessor.
        candidate_predecessor: Option<RetentionRootDigest>,
    },
    /// Exact retry disagreed with the current manifest selection.
    CurrentManifestEntryMismatch {
        /// Candidate namespace.
        namespace: RetentionNamespaceDigest,
        /// Generation selected by the current manifest.
        current_generation: RootGeneration,
        /// Root digest selected by the current manifest.
        current_digest: RetentionRootDigest,
        /// Candidate root generation.
        candidate_generation: RootGeneration,
        /// Candidate root digest.
        candidate_digest: RetentionRootDigest,
    },
    /// A namespace absent from the manifest carried a noninitial candidate.
    UnexpectedNamespaceSuccessor {
        /// Candidate namespace.
        namespace: RetentionNamespaceDigest,
        /// Noninitial candidate generation.
        generation: RootGeneration,
        /// Candidate-declared predecessor.
        predecessor: Option<RetentionRootDigest>,
    },
    /// Internal manifest update index escaped the admitted entry range.
    ManifestEntryIndex {
        /// Attempted insertion or replacement index.
        index: usize,
        /// Current manifest entry count.
        entry_count: usize,
    },
    /// Global liveness generation could not advance.
    LivenessGeneration {
        /// Preserved checked-generation refusal.
        source: LivenessGenerationError,
    },
    /// A manifest-selected root generation could not advance.
    RootGeneration {
        /// Preserved checked-generation refusal.
        source: RootGenerationError,
    },
    /// Bounded successor-entry allocation was refused.
    EntryAllocation {
        /// Preserved allocation refusal.
        source: TryReserveError,
    },
    /// Successor manifest semantics were refused.
    Manifest {
        /// Preserved semantic refusal.
        source: RetentionManifestError,
    },
    /// Canonical successor manifest encoding failed.
    ManifestEncoding {
        /// Preserved encoding refusal.
        source: RetentionManifestEncodeError,
    },
    /// Host length could not fit the protocol length domain.
    ManifestLengthOverflow {
        /// Observed canonical byte length.
        observed: usize,
    },
    /// Canonical successor manifest length was refused.
    ManifestLength {
        /// Preserved typed-length refusal.
        source: RetentionManifestLengthError,
    },
    /// Successor head semantics were refused.
    Head {
        /// Preserved semantic refusal.
        source: RetentionHeadError,
    },
}

impl fmt::Display for RetentionPublicationPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentManifestRequired { .. } => {
                formatter.write_str("retention retry requires a current global manifest")
            }
            Self::CurrentManifestEntryMissing { .. } => {
                formatter.write_str("retention retry root is absent from the current manifest")
            }
            Self::ManifestSuccessorMismatch { .. } => {
                formatter.write_str("retention manifest entry disagrees with the candidate root")
            }
            Self::CurrentManifestEntryMismatch { .. } => {
                formatter.write_str("retention retry disagrees with the current manifest entry")
            }
            Self::UnexpectedNamespaceSuccessor { .. } => {
                formatter.write_str("new retention namespace candidate is not generation one")
            }
            Self::ManifestEntryIndex { index, entry_count } => write!(
                formatter,
                "retention manifest update index {index} exceeds {entry_count} entries"
            ),
            Self::LivenessGeneration { source } => write!(formatter, "{source}"),
            Self::RootGeneration { source } => write!(formatter, "{source}"),
            Self::EntryAllocation { .. } => {
                formatter.write_str("retention successor entry allocation failed")
            }
            Self::Manifest { .. } => {
                formatter.write_str("retention successor manifest admission failed")
            }
            Self::ManifestEncoding { .. } => {
                formatter.write_str("retention successor manifest encoding failed")
            }
            Self::ManifestLengthOverflow { observed } => write!(
                formatter,
                "retention manifest byte length {observed} exceeds the protocol integer domain"
            ),
            Self::ManifestLength { .. } => {
                formatter.write_str("retention successor manifest length admission failed")
            }
            Self::Head { .. } => formatter.write_str("retention successor head admission failed"),
        }
    }
}

impl Error for RetentionPublicationPreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LivenessGeneration { source } => Some(source),
            Self::RootGeneration { source } => Some(source),
            Self::EntryAllocation { source } => Some(source),
            Self::Manifest { source } => Some(source),
            Self::ManifestEncoding { source } => Some(source),
            Self::ManifestLength { source } => Some(source),
            Self::Head { source } => Some(source),
            Self::CurrentManifestRequired { .. }
            | Self::CurrentManifestEntryMissing { .. }
            | Self::ManifestSuccessorMismatch { .. }
            | Self::CurrentManifestEntryMismatch { .. }
            | Self::UnexpectedNamespaceSuccessor { .. }
            | Self::ManifestEntryIndex { .. }
            | Self::ManifestLengthOverflow { .. } => None,
        }
    }
}
