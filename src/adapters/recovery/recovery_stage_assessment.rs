//! This module owns name-selected semantic recovery-stage assessments.

use super::{
    RecoveryCatalogStage, RecoveryNextHeadStage, RecoverySegmentStage, RecoveryStageEvidence,
};

/// Read-only semantic assessment of fingerprint-bound fixed-stage bytes.
#[must_use]
pub enum RecoveryStageAssessment<'a> {
    /// `staging/current.seg` semantic state.
    Segment {
        /// Exact evidence matched before semantic classification.
        evidence: RecoveryStageEvidence,
        /// Validated segment-stage semantic state.
        state: RecoverySegmentStage<'a>,
    },
    /// `staging/current.cat` semantic state.
    Catalog {
        /// Exact evidence matched before semantic classification.
        evidence: RecoveryStageEvidence,
        /// Validated catalog-stage semantic state.
        state: RecoveryCatalogStage<'a>,
    },
    /// Root `head.next` semantic state.
    NextHead {
        /// Exact evidence matched before semantic classification.
        evidence: RecoveryStageEvidence,
        /// Validated next-head semantic state.
        state: RecoveryNextHeadStage<'a>,
    },
}

impl RecoveryStageAssessment<'_> {
    /// Returns the exact evidence matched before semantic classification.
    pub const fn evidence(&self) -> RecoveryStageEvidence {
        match self {
            Self::Segment { evidence, .. }
            | Self::Catalog { evidence, .. }
            | Self::NextHead { evidence, .. } => *evidence,
        }
    }
}
