//! This module owns next-head finalization planning refusals.

use std::error::Error;
use std::fmt;

use super::{CatalogTransitionError, RecoveryStage};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Why an assessed `head.next` cannot enter finalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryNextHeadFinalizationPlanError {
    /// The assessment belongs to a fixed stage other than `head.next`.
    NotNextHead {
        /// Fixed stage that requires a different recovery protocol.
        stage: RecoveryStage,
    },
    /// The `head.next` bytes are incomplete.
    NotComplete,
    /// The complete transitive snapshot does not match the assessed head.
    SnapshotCoordinate {
        /// Generation named by the assessed head.
        expected_generation: CatalogGeneration,
        /// Catalog length named by the assessed head.
        expected_length: CatalogLength,
        /// Catalog digest named by the assessed head.
        expected_digest: CatalogDigest,
        /// Generation pinned by the complete snapshot.
        observed_generation: CatalogGeneration,
        /// Catalog length pinned by the complete snapshot.
        observed_length: CatalogLength,
        /// Catalog digest pinned by the complete snapshot.
        observed_digest: CatalogDigest,
    },
    /// An uninitialized root can only admit generation one.
    InitialGeneration {
        /// Candidate generation observed in the complete snapshot.
        observed: CatalogGeneration,
    },
    /// The candidate is not the exact successor of the expected current head.
    Transition {
        /// Exact generation or predecessor refusal.
        source: CatalogTransitionError,
    },
}

impl fmt::Display for RecoveryNextHeadFinalizationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotNextHead { stage } => {
                write!(formatter, "{stage} is not the recovery next head")
            }
            Self::NotComplete => formatter.write_str("recovery next head is incomplete"),
            Self::SnapshotCoordinate {
                expected_generation,
                expected_length,
                expected_digest,
                observed_generation,
                observed_length,
                observed_digest,
            } => write!(
                formatter,
                "recovery next-head coordinate generation {} length {} digest {:?} does not match snapshot generation {} length {} digest {:?}",
                expected_generation.get(),
                expected_length.get(),
                expected_digest,
                observed_generation.get(),
                observed_length.get(),
                observed_digest
            ),
            Self::InitialGeneration { observed } => write!(
                formatter,
                "uninitialized recovery requires generation 1, observed {}",
                observed.get()
            ),
            Self::Transition { source } => {
                write!(
                    formatter,
                    "recovery next head is not an exact successor: {source}"
                )
            }
        }
    }
}

impl Error for RecoveryNextHeadFinalizationPlanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transition { source } => Some(source),
            Self::NotNextHead { .. }
            | Self::NotComplete
            | Self::SnapshotCoordinate { .. }
            | Self::InitialGeneration { .. } => None,
        }
    }
}
