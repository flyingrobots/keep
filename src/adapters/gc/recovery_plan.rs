//! This boundary module owns the pure classification of GC residue into
//! the one lawful recovery, or a typed ambiguity.

use std::fmt;

use super::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, GcExecutionPhase, GcExecutionPoint,
    GcResidue, GcRetirementIntentDecodeError, GcRetirementReceiptDecodeError,
};

/// One of the two fixed stages GC execution retains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcFixedStage {
    /// `gc/intent.next`.
    Intent,
    /// `gc/receipt.next`.
    Receipt,
}

/// What restart does with the residue it found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcRecoveryPlan {
    /// Nothing is in progress and no retirement has completed.
    Idle,
    /// The last retirement completed: `gc/receipt` alone, exactly decodable.
    Complete,
    /// A truncated stage written before any authority is discarded, and
    /// `gc` synchronized; nothing else changes.
    DiscardStage {
        /// The stage to discard.
        stage: GcFixedStage,
    },
    /// Execution resumes at `from` with the durable intent's candidates.
    Resume {
        /// The point to resume at.
        from: GcExecutionPoint,
    },
}

/// Residue that admits no lawful recovery. Nothing is unlinked or repaired.
#[derive(Debug)]
pub enum GcRecoveryAmbiguity {
    /// `gc/intent` does not decode.
    IntentUndecodable {
        /// The exact refusal.
        source: GcRetirementIntentDecodeError,
    },
    /// `gc/receipt` neither completes the durable intent nor decodes as
    /// the retirement the intent succeeds.
    ReceiptUndecodable {
        /// The exact refusal against the intent.
        source: GcRetirementReceiptDecodeError,
    },
    /// A receipt stage exists without the intent it completes.
    ReceiptWithoutIntent,
    /// A stage holds bytes other than the record it stages.
    StageDiffers {
        /// The stage.
        stage: GcFixedStage,
    },
    /// A stage is longer than its record.
    StageOverlong {
        /// The stage.
        stage: GcFixedStage,
    },
    /// A candidate is absent after a present one: no prefix explains it.
    AbsentOutOfOrder {
        /// The first absent candidate index.
        absent: usize,
        /// A later present candidate index.
        present: usize,
    },
    /// A receipt or receipt stage exists while a candidate is still present.
    ReceiptBeforeRetirement,
}

impl fmt::Display for GcRecoveryAmbiguity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntentUndecodable { .. } => formatter.write_str("gc/intent does not decode"),
            Self::ReceiptUndecodable { .. } => {
                formatter.write_str("gc/receipt neither completes nor precedes the intent")
            }
            Self::ReceiptWithoutIntent => {
                formatter.write_str("gc/receipt.next exists without the intent it completes")
            }
            Self::StageDiffers { stage } => {
                write!(formatter, "gc {stage:?} stage holds other bytes")
            }
            Self::StageOverlong { stage } => {
                write!(formatter, "gc {stage:?} stage is longer than its record")
            }
            Self::AbsentOutOfOrder { absent, present } => write!(
                formatter,
                "candidate {absent} is absent while later candidate {present} is present"
            ),
            Self::ReceiptBeforeRetirement => {
                formatter.write_str("gc receipt exists while a candidate is still present")
            }
        }
    }
}

impl std::error::Error for GcRecoveryAmbiguity {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IntentUndecodable { source } => Some(source),
            Self::ReceiptUndecodable { source } => Some(source),
            _ => None,
        }
    }
}

/// How `gc/receipt` relates to the durable intent.
enum ReceiptRelation {
    /// No receipt.
    Absent,
    /// The receipt of the retirement this intent succeeds.
    Prior,
    /// The receipt that completes this intent.
    Complete,
}

/// Plans the one lawful recovery of `residue`.
///
/// The table follows `gc.md` and ADR-0009: idle, active (every candidate
/// present), partial (one exact absent prefix), completion pending (every
/// candidate absent, no receipt for the intent), receipt transition (exact
/// intent and its exact receipt), and complete (exact receipt only). A
/// receipt whose generation the intent succeeds is the prior retirement's
/// and constrains nothing. Every other residue is a typed ambiguity.
///
/// # Errors
///
/// Returns [`GcRecoveryAmbiguity`] when the residue matches no row.
pub fn plan_gc_recovery(residue: &GcResidue) -> Result<GcRecoveryPlan, GcRecoveryAmbiguity> {
    let Some(intent_bytes) = residue.intent.as_deref() else {
        return plan_before_durable_intent(residue);
    };
    let intent = AdmittedGcRetirementIntent::decode(intent_bytes)
        .map_err(|source| GcRecoveryAmbiguity::IntentUndecodable { source })?;
    if let Some(stage) = residue.intent_stage.as_deref() {
        return if stage == intent_bytes {
            Ok(resume(GcExecutionPhase::SynchronizeGcAfterIntent))
        } else {
            Err(GcRecoveryAmbiguity::StageDiffers {
                stage: GcFixedStage::Intent,
            })
        };
    }
    let absent_prefix = absent_prefix(&residue.candidates_present)?;
    let count = residue.candidates_present.len();
    match receipt_relation(residue.receipt.as_deref(), &intent)? {
        ReceiptRelation::Complete if absent_prefix != count => {
            Err(GcRecoveryAmbiguity::ReceiptBeforeRetirement)
        }
        ReceiptRelation::Complete if residue.receipt_stage.is_some() => {
            Err(GcRecoveryAmbiguity::StageDiffers {
                stage: GcFixedStage::Receipt,
            })
        }
        ReceiptRelation::Complete => Ok(resume(GcExecutionPhase::SynchronizeGcAfterReceipt)),
        ReceiptRelation::Absent | ReceiptRelation::Prior => {
            plan_after_unlinks(residue, &intent, absent_prefix, count)
        }
    }
}

/// Rows three through five: the intent is durable and no receipt completes it.
fn plan_after_unlinks(
    residue: &GcResidue,
    intent: &AdmittedGcRetirementIntent<'_>,
    absent_prefix: usize,
    count: usize,
) -> Result<GcRecoveryPlan, GcRecoveryAmbiguity> {
    if let Some(stage) = residue.receipt_stage.as_deref() {
        if absent_prefix != count {
            return Err(GcRecoveryAmbiguity::ReceiptBeforeRetirement);
        }
        return match AdmittedGcRetirementReceipt::decode(stage, intent) {
            Ok(_complete) => Ok(resume(GcExecutionPhase::SynchronizeReceiptStage)),
            Err(GcRetirementReceiptDecodeError::WrongLength { expected, observed })
                if observed < expected =>
            {
                Ok(GcRecoveryPlan::DiscardStage {
                    stage: GcFixedStage::Receipt,
                })
            }
            Err(GcRetirementReceiptDecodeError::WrongLength { .. }) => {
                Err(GcRecoveryAmbiguity::StageOverlong {
                    stage: GcFixedStage::Receipt,
                })
            }
            Err(_) => Err(GcRecoveryAmbiguity::StageDiffers {
                stage: GcFixedStage::Receipt,
            }),
        };
    }
    Ok(GcRecoveryPlan::Resume {
        from: if absent_prefix == count {
            GcExecutionPoint::at(GcExecutionPhase::WriteReceiptStage)
        } else if absent_prefix == 0 {
            GcExecutionPoint::at(GcExecutionPhase::SynchronizeGcAfterIntentCleanup)
        } else {
            GcExecutionPoint {
                phase: GcExecutionPhase::UnlinkCandidate,
                candidate: absent_prefix,
            }
        },
    })
}

/// Rows one, two, and six: no durable intent.
fn plan_before_durable_intent(residue: &GcResidue) -> Result<GcRecoveryPlan, GcRecoveryAmbiguity> {
    if residue.receipt_stage.is_some() {
        return Err(GcRecoveryAmbiguity::ReceiptWithoutIntent);
    }
    let Some(stage) = residue.intent_stage.as_deref() else {
        return residue
            .receipt
            .as_deref()
            .map_or(Ok(GcRecoveryPlan::Idle), |receipt| {
                AdmittedGcRetirementReceipt::decode_unbound(receipt)
                    .map(|_prior| GcRecoveryPlan::Complete)
                    .map_err(|source| GcRecoveryAmbiguity::ReceiptUndecodable { source })
            });
    };
    match AdmittedGcRetirementIntent::decode(stage) {
        Ok(_complete) => Ok(resume(GcExecutionPhase::SynchronizeIntentStage)),
        Err(GcRetirementIntentDecodeError::Truncated { .. }) => Ok(GcRecoveryPlan::DiscardStage {
            stage: GcFixedStage::Intent,
        }),
        Err(GcRetirementIntentDecodeError::TrailingData { .. }) => {
            Err(GcRecoveryAmbiguity::StageOverlong {
                stage: GcFixedStage::Intent,
            })
        }
        Err(_) => Err(GcRecoveryAmbiguity::StageDiffers {
            stage: GcFixedStage::Intent,
        }),
    }
}

fn receipt_relation(
    receipt: Option<&[u8]>,
    intent: &AdmittedGcRetirementIntent<'_>,
) -> Result<ReceiptRelation, GcRecoveryAmbiguity> {
    let Some(receipt) = receipt else {
        return Ok(ReceiptRelation::Absent);
    };
    let refusal = match AdmittedGcRetirementReceipt::decode(receipt, intent) {
        Ok(_complete) => return Ok(ReceiptRelation::Complete),
        Err(refusal) => refusal,
    };
    let generation = intent.intent().coordinates().generation.get();
    match AdmittedGcRetirementReceipt::decode_unbound(receipt) {
        Ok(prior) if prior.generation().get().checked_add(1) == Some(generation) => {
            Ok(ReceiptRelation::Prior)
        }
        Ok(_) | Err(_) => Err(GcRecoveryAmbiguity::ReceiptUndecodable { source: refusal }),
    }
}

/// The length of the absent prefix, refusing an absent candidate after a
/// present one.
fn absent_prefix(present: &[bool]) -> Result<usize, GcRecoveryAmbiguity> {
    let prefix = present.iter().take_while(|present| !**present).count();
    if let Some(absent) = present
        .iter()
        .skip(prefix)
        .position(|present| !*present)
        .map(|offset| offset.saturating_add(prefix))
    {
        return Err(GcRecoveryAmbiguity::AbsentOutOfOrder {
            absent,
            present: prefix,
        });
    }
    Ok(prefix)
}

const fn resume(phase: GcExecutionPhase) -> GcRecoveryPlan {
    GcRecoveryPlan::Resume {
        from: GcExecutionPoint::at(phase),
    }
}

/// Whether a residue means a complete retirement: exact receipt only.
#[must_use]
pub const fn is_complete(residue: &GcResidue) -> bool {
    residue.intent.is_none()
        && residue.intent_stage.is_none()
        && residue.receipt_stage.is_none()
        && residue.receipt.is_some()
}
