//! This boundary module owns ordered migration recovery: observe, plan,
//! adopt, discard, resume.

use std::error::Error;
use std::fmt;
use std::io;

use super::migration_resumption::resume_store_migration;
use super::{
    AdmittedStoreMigrationIntent, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    StoreMigrationError, StoreMigrationFixedStage, StoreMigrationRecoveryAmbiguity,
    StoreMigrationRecoveryPlan, StoreMigrationRecoveryStorage, StoreMigrationResidue,
    plan_store_migration_recovery,
};

/// What one recovery found and did.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreMigrationRecoveryReceipt {
    plan: StoreMigrationRecoveryPlan,
    observed_prefix: super::StoreMigrationNamespacePrefix,
    intent_digest: Option<super::StoreMigrationIntentDigest>,
    published: Option<CanonicalStoreMigrationReceipt>,
}

impl StoreMigrationRecoveryReceipt {
    /// Returns the plan the residue admitted.
    pub const fn plan(&self) -> StoreMigrationRecoveryPlan {
        self.plan
    }

    /// Returns the exact namespace prefix observed before any recovery writes.
    pub const fn observed_namespace_prefix(&self) -> super::StoreMigrationNamespacePrefix {
        self.observed_prefix
    }

    /// Returns the intent bound by this recovery, including complete observations.
    ///
    /// An untouched version-one observation has no migration intent. Resumed
    /// recovery binds persisted intent bytes, or the newly published intent
    /// after discarding an incomplete pre-effect stage.
    pub const fn intent_digest(&self) -> Option<super::StoreMigrationIntentDigest> {
        self.intent_digest
    }

    /// Returns the migration receipt this recovery published, when it ran
    /// the forward protocol to completion.
    pub const fn published(&self) -> Option<&CanonicalStoreMigrationReceipt> {
        self.published.as_ref()
    }

    /// Lists every forward phase this successful recovery executed, in order.
    ///
    /// A version-one admission or already-complete observation executed no
    /// forward phases. Any pre-effect discard is recorded separately in
    /// [`Self::plan`]. No allocation or I/O occurs during iteration.
    pub fn executed_phases(&self) -> impl Iterator<Item = super::StoreMigrationPhase> {
        let first = match self.plan {
            StoreMigrationRecoveryPlan::Resume { resume }
            | StoreMigrationRecoveryPlan::DiscardStage { resume, .. } => Some(resume),
            StoreMigrationRecoveryPlan::VersionOne | StoreMigrationRecoveryPlan::Complete => None,
        };
        super::StoreMigrationPhase::ALL
            .into_iter()
            .skip_while(move |phase| Some(*phase) != first)
    }
}

/// Failure to recover one interrupted migration.
#[derive(Debug)]
pub enum StoreMigrationRecoveryError {
    /// Current writer authority no longer reproduces the caller's expected intent.
    CurrentVerification {
        /// Preserved current-state refusal or operational failure.
        source: io::Error,
    },
    /// The residue could not be observed.
    Observation {
        /// Preserved storage failure.
        source: io::Error,
    },
    /// The residue matches no lawful recovery row.
    Ambiguity {
        /// The exact conflict.
        source: StoreMigrationRecoveryAmbiguity,
    },
    /// The exact stage and canonical handles could not be adopted.
    Adoption {
        /// Preserved storage failure.
        source: io::Error,
    },
    /// The incomplete pre-effect stage could not be removed.
    Discard {
        /// The stage.
        stage: StoreMigrationFixedStage,
        /// Preserved storage failure.
        source: io::Error,
    },
    /// A resumed forward phase refused.
    Resumption {
        /// Preserved phase failure.
        source: StoreMigrationError,
    },
}

/// Recovers one interrupted migration under writer authority.
///
/// Current writer authority first revalidates `expected`. The residue is then
/// observed once and planned against `expected` (the intent the
/// version-1 store derives today, compared on every restart-stable
/// coordinate), and either admitted as version 1, reported complete, or
/// driven through the remaining forward phases with the persisted intent,
/// never the freshly derived one. Nothing is truncated, replaced, or
/// repaired; an incomplete pre-effect stage is the only artifact removed.
///
/// # Errors
///
/// Returns [`StoreMigrationRecoveryError`] at the exact boundary that refused.
///
/// Resumption is internal: external callers cannot bypass recovery admission.
///
/// ```compile_fail
/// use keep::resume_store_migration;
/// ```
pub fn recover_store_migration(
    storage: &mut impl StoreMigrationRecoveryStorage,
    expected: &CanonicalStoreMigrationIntent,
) -> Result<StoreMigrationRecoveryReceipt, StoreMigrationRecoveryError> {
    storage
        .verify_current(expected)
        .map_err(|source| StoreMigrationRecoveryError::CurrentVerification { source })?;
    let residue = storage
        .observe_residue()
        .map_err(|source| StoreMigrationRecoveryError::Observation { source })?;
    let expected_admitted =
        AdmittedStoreMigrationIntent::decode(expected.encoded()).map_err(|source| {
            StoreMigrationRecoveryError::Observation {
                source: io::Error::new(io::ErrorKind::InvalidData, source),
            }
        })?;
    let plan = plan_store_migration_recovery(&expected_admitted, &residue)
        .map_err(|source| StoreMigrationRecoveryError::Ambiguity { source })?;
    let observed_prefix = super::StoreMigrationNamespacePrefix::observe(&residue)
        .map_err(|source| StoreMigrationRecoveryError::Ambiguity { source })?;
    let persisted = persisted_intent(&residue, expected)?;
    let intent_digest =
        (plan != StoreMigrationRecoveryPlan::VersionOne).then(|| persisted.digest());
    let resume = match plan {
        StoreMigrationRecoveryPlan::VersionOne | StoreMigrationRecoveryPlan::Complete => {
            return Ok(StoreMigrationRecoveryReceipt {
                plan,
                observed_prefix,
                intent_digest,
                published: None,
            });
        }
        StoreMigrationRecoveryPlan::DiscardStage { stage, resume } => {
            storage
                .adopt_residue(&residue, &persisted)
                .map_err(|source| StoreMigrationRecoveryError::Adoption { source })?;
            storage
                .discard_stage(stage)
                .map_err(|source| StoreMigrationRecoveryError::Discard { stage, source })?;
            resume
        }
        StoreMigrationRecoveryPlan::Resume { resume } => {
            storage
                .adopt_residue(&residue, &persisted)
                .map_err(|source| StoreMigrationRecoveryError::Adoption { source })?;
            resume
        }
    };
    let published = resume_store_migration(storage, &persisted, resume)
        .map_err(|source| StoreMigrationRecoveryError::Resumption { source })?;
    Ok(StoreMigrationRecoveryReceipt {
        plan,
        observed_prefix,
        intent_digest,
        published: Some(published),
    })
}

/// The intent the resumed phases must publish: the durable one when it
/// exists, else the exact staged one, else the freshly derived one.
fn persisted_intent(
    residue: &StoreMigrationResidue,
    expected: &CanonicalStoreMigrationIntent,
) -> Result<CanonicalStoreMigrationIntent, StoreMigrationRecoveryError> {
    let staged = residue
        .intent
        .as_deref()
        .or(residue.intent_stage.as_deref());
    let Some(bytes) = staged else {
        return Ok(expected.clone());
    };
    match AdmittedStoreMigrationIntent::decode(bytes) {
        Ok(admitted) => Ok(CanonicalStoreMigrationIntent::from_admitted(&admitted)),
        Err(_) if residue.intent.is_none() => Ok(expected.clone()),
        Err(source) => Err(StoreMigrationRecoveryError::Observation {
            source: io::Error::new(io::ErrorKind::InvalidData, source),
        }),
    }
}

impl fmt::Display for StoreMigrationRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentVerification { .. } => {
                formatter.write_str("current migration authority verification failed")
            }
            Self::Observation { .. } => formatter.write_str("migration residue observation failed"),
            Self::Ambiguity { source } => {
                write!(formatter, "migration residue is ambiguous: {source}")
            }
            Self::Adoption { .. } => formatter.write_str("migration residue adoption failed"),
            Self::Discard { stage, .. } => {
                write!(formatter, "migration {stage:?} stage discard failed")
            }
            Self::Resumption { source } => write!(formatter, "resumed migration failed: {source}"),
        }
    }
}

impl Error for StoreMigrationRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentVerification { source }
            | Self::Observation { source }
            | Self::Adoption { source }
            | Self::Discard { source, .. } => Some(source),
            Self::Ambiguity { source } => Some(source),
            Self::Resumption { source } => Some(source),
        }
    }
}
