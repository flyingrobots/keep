//! This boundary module owns typed refusals of filesystem GC execution and
//! recovery.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    GcExecutionError, GcIntentDerivationError, GcLivenessObservationError, GcPlanError,
    GcRecoveryAmbiguity, GcRecoveryPlan, GcRetirementIntentEncodeError,
};
use crate::GcGenerationError;
use crate::adapters::FilesystemRetentionSnapshotError;

/// Why filesystem GC refused to execute or recover.
#[derive(Debug)]
pub enum FilesystemGcError {
    /// A read, open, or synchronization outside any phase failed.
    Observe {
        /// The exact failure.
        source: io::Error,
    },
    /// A reader holds the shared fence; retirement does not wait on it.
    ReadersActive,
    /// `gc` holds residue that recovery must resolve before a new intent.
    RecoveryRequired {
        /// What recovery would do with it.
        plan: GcRecoveryPlan,
    },
    /// The residue admits no lawful recovery.
    Ambiguity(GcRecoveryAmbiguity),
    /// The store no longer matches the plan: re-observed liveness planned
    /// differently, so the plan's evidence is stale.
    PlanStale,
    /// The plan names no candidate; nothing to do is not an intent.
    NothingToRetire,
    /// The reopened view refused.
    Snapshot(Box<FilesystemRetentionSnapshotError>),
    /// Re-observing liveness refused.
    Liveness(Box<GcLivenessObservationError>),
    /// Re-planning refused.
    Plan(GcPlanError),
    /// The plan yielded no intent.
    Intent(GcIntentDerivationError),
    /// The intent refused to encode.
    Encode(GcRetirementIntentEncodeError),
    /// The prior receipt's generation has no successor.
    Generation(GcGenerationError),
    /// A phase refused; the completed phases' effects remain for recovery.
    Execute(GcExecutionError),
}

impl fmt::Display for FilesystemGcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Observe { .. } => formatter.write_str("GC observation failed"),
            Self::ReadersActive => {
                formatter.write_str("readers hold the fence; GC refuses to wait")
            }
            Self::RecoveryRequired { plan } => {
                write!(formatter, "gc holds residue requiring recovery: {plan:?}")
            }
            Self::Ambiguity(source) => write!(formatter, "GC residue is ambiguous: {source}"),
            Self::PlanStale => formatter.write_str("the store no longer matches the GC plan"),
            Self::NothingToRetire => formatter.write_str("the GC plan names no candidate"),
            Self::Snapshot(_) => formatter.write_str("GC could not reopen the store view"),
            Self::Liveness(_) => formatter.write_str("GC could not re-observe liveness"),
            Self::Plan(source) => write!(formatter, "GC re-planning refused: {source}"),
            Self::Intent(source) => write!(formatter, "GC intent derivation refused: {source}"),
            Self::Encode(source) => write!(formatter, "GC intent encoding refused: {source}"),
            Self::Generation(source) => write!(formatter, "GC generation refused: {source}"),
            Self::Execute(source) => write!(formatter, "GC execution refused: {source}"),
        }
    }
}

impl Error for FilesystemGcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observe { source } => Some(source),
            Self::Ambiguity(source) => Some(source),
            Self::Snapshot(source) => Some(source.as_ref()),
            Self::Liveness(source) => Some(source.as_ref()),
            Self::Plan(source) => Some(source),
            Self::Intent(source) => Some(source),
            Self::Encode(source) => Some(source),
            Self::Generation(source) => Some(source),
            Self::Execute(source) => Some(source),
            Self::ReadersActive
            | Self::RecoveryRequired { .. }
            | Self::PlanStale
            | Self::NothingToRetire => None,
        }
    }
}

pub(super) const fn observe(source: io::Error) -> FilesystemGcError {
    FilesystemGcError::Observe { source }
}
