//! This module owns typed semantic retention manifest failures.

use std::{error::Error, fmt};

use super::{LivenessGeneration, RetentionManifestDigest, RetentionNamespaceDigest};

/// Failure to construct one canonical semantic retention manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionManifestError {
    /// Generation one carried an impossible predecessor.
    InitialGenerationHasPredecessor {
        /// Observed predecessor digest.
        observed: RetentionManifestDigest,
    },
    /// A successor generation omitted its required predecessor.
    MissingPredecessor {
        /// Successor generation lacking a predecessor.
        generation: LivenessGeneration,
    },
    /// The caller supplied too many namespace entries.
    EntryCountExceeded {
        /// Fixed maximum entry count.
        maximum: u32,
        /// Observed entry count.
        observed: usize,
    },
    /// The caller supplied one namespace more than once.
    DuplicateNamespace {
        /// Exact duplicated namespace digest.
        namespace: RetentionNamespaceDigest,
    },
}

impl fmt::Display for RetentionManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitialGenerationHasPredecessor { observed } => write!(
                formatter,
                "initial retention manifest has predecessor {:?}",
                observed.as_bytes()
            ),
            Self::MissingPredecessor { generation } => write!(
                formatter,
                "retention manifest generation {} requires a predecessor",
                generation.get()
            ),
            Self::EntryCountExceeded { maximum, observed } => write!(
                formatter,
                "retention manifest has {observed} entries; maximum is {maximum}"
            ),
            Self::DuplicateNamespace { namespace } => write!(
                formatter,
                "retention manifest repeats namespace {:?}",
                namespace.as_bytes()
            ),
        }
    }
}

impl Error for RetentionManifestError {}
