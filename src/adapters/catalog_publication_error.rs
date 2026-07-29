//! Catalog-generation publication failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    CatalogAdmissionError, CatalogPublicationPhase, CatalogSnapshotError,
    PublicationHeadDecodeError, SegmentDigest,
};
use crate::{CatalogGeneration, CatalogTransitionError};

/// Failure before one catalog generation becomes durably visible.
#[derive(Debug)]
pub enum CatalogPublicationError {
    /// An uninitialized store was paired with a later catalog generation.
    InitialGeneration {
        /// Candidate generation that cannot initialize a store.
        observed: CatalogGeneration,
    },
    /// A published store was paired with a non-successor candidate.
    Transition {
        /// Exact successor-law refusal.
        source: CatalogTransitionError,
    },
    /// The selected segment stage was absent from the admitted segment set.
    StagedSegmentNotAdmitted {
        /// Exact physical segment digest selected for publication.
        segment_digest: SegmentDigest,
    },
    /// Catalog locations failed complete segment-record admission.
    CatalogAdmission {
        /// Preserved admission refusal.
        source: Box<CatalogAdmissionError>,
    },
    /// Generated next-head bytes failed canonical decoder verification.
    HeadVerification {
        /// Preserved head decoder refusal.
        source: PublicationHeadDecodeError,
    },
    /// The verified head and admitted catalog did not bind exactly.
    SnapshotAdmission {
        /// Preserved snapshot refusal.
        source: CatalogSnapshotError,
    },
    /// One exact storage transition failed.
    Storage {
        /// Transition attempted.
        phase: CatalogPublicationPhase,
        /// Preserved filesystem source.
        source: io::Error,
    },
}

impl CatalogPublicationError {
    pub(super) const fn storage(phase: CatalogPublicationPhase, source: io::Error) -> Self {
        Self::Storage { phase, source }
    }
}

impl fmt::Display for CatalogPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitialGeneration { observed } => write!(
                formatter,
                "initial catalog generation must be 1, observed {}",
                observed.get()
            ),
            Self::Transition { .. } => {
                formatter.write_str("catalog publication candidate is not the exact successor")
            }
            Self::StagedSegmentNotAdmitted { .. } => {
                formatter.write_str("staged segment is absent from the admitted segment set")
            }
            Self::CatalogAdmission { .. } => {
                formatter.write_str("catalog publication admission failed")
            }
            Self::HeadVerification { .. } => {
                formatter.write_str("generated publication head verification failed")
            }
            Self::SnapshotAdmission { .. } => {
                formatter.write_str("publication snapshot admission failed")
            }
            Self::Storage { phase, .. } => write!(formatter, "publication {phase} failed"),
        }
    }
}

impl Error for CatalogPublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transition { source } => Some(source),
            Self::CatalogAdmission { source } => Some(source),
            Self::HeadVerification { source } => Some(source),
            Self::SnapshotAdmission { source } => Some(source),
            Self::Storage { source, .. } => Some(source),
            Self::InitialGeneration { .. } | Self::StagedSegmentNotAdmitted { .. } => None,
        }
    }
}
