//! This module owns the complete evidence retention recovery plans from.

use super::{
    ObservedRetentionState, RetentionHeadStageAssessment, RetentionManifestStageAssessment,
    RetentionRootStageAssessment,
};

/// Whether an immutable pool already holds the entry a complete stage names.
///
/// The observation is meaningful only for a `Complete` stage: a truncated or
/// corrupt stage names no canonical entry, and the adapter reports `Absent`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPoolEntryObservation {
    /// No entry exists under the canonical name.
    Absent,
    /// The entry exists with exactly the stage's bytes.
    Identical,
    /// The entry exists with other bytes or another kind.
    Different,
}

/// The three fixed-stage assessments read at restart.
#[derive(Debug)]
pub struct RetentionStageAssessments<'bytes> {
    /// Assessment of `retention/root.next`.
    pub root: RetentionRootStageAssessment<'bytes>,
    /// Assessment of `retention/manifest.next`.
    pub manifest: RetentionManifestStageAssessment<'bytes>,
    /// Assessment of `retention/head.next`.
    pub head: RetentionHeadStageAssessment<'bytes>,
}

/// Whether each immutable pool holds the entry its complete stage names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionPoolObservations {
    /// The root pool entry the complete root stage names.
    pub root: RetentionPoolEntryObservation,
    /// The manifest pool entry the complete manifest stage names.
    pub manifest: RetentionPoolEntryObservation,
}

/// Everything restart observed before planning retention recovery.
///
/// The evidence is read under exclusive writer authority and performs no
/// mutation; planning over it is pure.
#[derive(Debug)]
pub struct RetentionRecoveryEvidence<'bytes, 'state> {
    current: Option<&'state ObservedRetentionState>,
    stages: RetentionStageAssessments<'bytes>,
    pools: RetentionPoolObservations,
}

impl<'bytes, 'state> RetentionRecoveryEvidence<'bytes, 'state> {
    /// Binds the observed current state, the three stage assessments, and the
    /// pool observations for the entries the complete stages name.
    #[must_use]
    pub const fn new(
        current: Option<&'state ObservedRetentionState>,
        stages: RetentionStageAssessments<'bytes>,
        pools: RetentionPoolObservations,
    ) -> Self {
        Self {
            current,
            stages,
            pools,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        Option<&'state ObservedRetentionState>,
        RetentionStageAssessments<'bytes>,
        RetentionPoolObservations,
    ) {
        (self.current, self.stages, self.pools)
    }

    /// The published head and manifest, or `None` when no head is published.
    #[must_use]
    pub const fn current(&self) -> Option<&'state ObservedRetentionState> {
        self.current
    }

    /// The three stage assessments.
    #[must_use]
    pub const fn stages(&self) -> &RetentionStageAssessments<'bytes> {
        &self.stages
    }

    /// The pool observations for the entries the complete stages name.
    #[must_use]
    pub const fn pools(&self) -> RetentionPoolObservations {
        self.pools
    }
}
