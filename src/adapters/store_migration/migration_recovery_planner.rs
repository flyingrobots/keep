//! This boundary module owns storage-independent migration recovery planning.
//!
//! The planner turns one observed residue into the one lawful response the
//! recovery table in `migration-recovery.md` prescribes, or into the exact
//! ambiguity that refuses it. It reads no storage and mutates nothing.

use super::StoreMigrationStageDecodeError as DecodeError;
use super::migration_recovery_ambiguity::StoreMigrationEffect as Effect;
use super::migration_recovery_residue::NAMESPACE_NAME_COUNT;
use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    FORMAT_MARKER_LENGTH, MIGRATION_INTENT_LENGTH, MIGRATION_RECEIPT_LENGTH,
    StoreMigrationFixedStage as Stage, StoreMigrationPhase as Phase,
    StoreMigrationRecoveryAmbiguity as Ambiguity, StoreMigrationRecoveryPlan as Plan,
    StoreMigrationResidue,
};

/// Plans the one lawful recovery of `residue` against the intent the
/// version-1 store derives today.
///
/// The expected intent is compared on every coordinate but the mount
/// identity, which is same-process evidence (see `recovery.md`, "Root
/// identity across restart").
///
/// # Errors
///
/// Returns [`StoreMigrationRecoveryAmbiguity`](super::StoreMigrationRecoveryAmbiguity)
/// when the residue matches no table row.
pub fn plan_store_migration_recovery(
    expected: &AdmittedStoreMigrationIntent<'_>,
    residue: &StoreMigrationResidue,
) -> Result<Plan, Ambiguity> {
    let Some(intent_bytes) = residue.intent.as_deref() else {
        return plan_before_durable_intent(expected, residue);
    };
    let intent = AdmittedStoreMigrationIntent::decode(intent_bytes)
        .map_err(|source| Ambiguity::IntentUndecodable { source })?;
    if !restart_stable_match(expected, &intent) {
        return Err(Ambiguity::IntentDiffers);
    }
    if let Some(stage) = residue.intent_stage.as_deref() {
        let later = if residue.has_namespace_effect() {
            Some(Effect::Namespace)
        } else if residue.marker_stage.is_some() || residue.marker.is_some() {
            Some(Effect::Marker)
        } else if residue.has_receipt_effect() {
            Some(Effect::Receipt)
        } else {
            None
        };
        if let Some(effect) = later {
            return Err(Ambiguity::StageAfterEffect {
                stage: Stage::Intent,
                effect,
            });
        }
        return if stage == intent_bytes {
            Ok(Plan::Resume {
                resume: Phase::SynchronizeRootAfterIntent,
            })
        } else {
            Err(Ambiguity::StageDiffers {
                stage: Stage::Intent,
            })
        };
    }
    plan_namespace(&intent, residue)
}

/// Rows one and two: nothing, or an intent stage alone.
fn plan_before_durable_intent(
    expected: &AdmittedStoreMigrationIntent<'_>,
    residue: &StoreMigrationResidue,
) -> Result<Plan, Ambiguity> {
    if residue.has_namespace_effect() {
        return Err(Ambiguity::EffectBeforeIntent {
            effect: Effect::Namespace,
        });
    }
    if residue.marker_stage.is_some() || residue.marker.is_some() {
        return Err(Ambiguity::EffectBeforeIntent {
            effect: Effect::Marker,
        });
    }
    if residue.has_receipt_effect() {
        return Err(Ambiguity::EffectBeforeIntent {
            effect: Effect::Receipt,
        });
    }
    let Some(stage) = residue.intent_stage.as_deref() else {
        return Ok(Plan::VersionOne);
    };
    match classify_stage(Stage::Intent, stage, MIGRATION_INTENT_LENGTH)? {
        StageShape::Incomplete => Ok(Plan::DiscardStage {
            stage: Stage::Intent,
            resume: Phase::WriteIntentStage,
        }),
        StageShape::Complete => {
            let staged = AdmittedStoreMigrationIntent::decode(stage).map_err(|source| {
                Ambiguity::StageUndecodable {
                    stage: Stage::Intent,
                    source: DecodeError::Intent { source },
                }
            })?;
            if restart_stable_match(expected, &staged) {
                Ok(Plan::Resume {
                    resume: Phase::SynchronizeIntentStage,
                })
            } else {
                Err(Ambiguity::StageDiffers {
                    stage: Stage::Intent,
                })
            }
        }
    }
}

/// Rows three and four: a durable intent, then `reader.lock` and the prefix.
fn plan_namespace(
    intent: &AdmittedStoreMigrationIntent<'_>,
    residue: &StoreMigrationResidue,
) -> Result<Plan, Ambiguity> {
    let extent = residue
        .namespace_extent()
        .map_err(|(absent, present)| Ambiguity::NamespaceOutOfOrder { absent, present })?;
    if extent < NAMESPACE_NAME_COUNT {
        if residue.marker_stage.is_some() || residue.marker.is_some() {
            return Err(Ambiguity::MarkerBeforeNamespace);
        }
        if residue.has_receipt_effect() {
            return Err(Ambiguity::ReceiptBeforeMarker);
        }
        return Ok(Plan::Resume {
            resume: if extent == 0 {
                Phase::SynchronizeRootAfterIntentCleanup
            } else {
                Phase::AdmitNamespacePrefix
            },
        });
    }
    plan_marker(intent, residue)
}

/// Rows five and six: the marker stage and the canonical marker.
fn plan_marker(
    intent: &AdmittedStoreMigrationIntent<'_>,
    residue: &StoreMigrationResidue,
) -> Result<Plan, Ambiguity> {
    let Some(marker_bytes) = residue.marker.as_deref() else {
        if residue.has_receipt_effect() {
            return Err(Ambiguity::ReceiptBeforeMarker);
        }
        let Some(stage) = residue.marker_stage.as_deref() else {
            return Ok(Plan::Resume {
                resume: Phase::SynchronizeRootAfterNamespace,
            });
        };
        return match classify_stage(Stage::Marker, stage, FORMAT_MARKER_LENGTH)? {
            StageShape::Incomplete => Ok(Plan::DiscardStage {
                stage: Stage::Marker,
                resume: Phase::WriteMarkerStage,
            }),
            StageShape::Complete => {
                AdmittedStoreFormatMarker::decode(stage)
                    .map(|_| ())
                    .map_err(|source| Ambiguity::StageUndecodable {
                        source: DecodeError::Marker { source },
                        stage: Stage::Marker,
                    })?;
                Ok(Plan::Resume {
                    resume: Phase::SynchronizeMarkerStage,
                })
            }
        };
    };
    let marker = AdmittedStoreFormatMarker::decode(marker_bytes)
        .map_err(|source| Ambiguity::MarkerUndecodable { source })?;
    if let Some(stage) = residue.marker_stage.as_deref() {
        if residue.has_receipt_effect() {
            return Err(Ambiguity::StageAfterEffect {
                stage: Stage::Marker,
                effect: Effect::Receipt,
            });
        }
        return if stage == marker_bytes {
            Ok(Plan::Resume {
                resume: Phase::SynchronizeRootAfterMarker,
            })
        } else {
            Err(Ambiguity::StageDiffers {
                stage: Stage::Marker,
            })
        };
    }
    plan_receipt(intent, &marker, residue)
}

/// Row seven: the receipt stage and the canonical receipt.
fn plan_receipt(
    intent: &AdmittedStoreMigrationIntent<'_>,
    marker: &AdmittedStoreFormatMarker<'_>,
    residue: &StoreMigrationResidue,
) -> Result<Plan, Ambiguity> {
    let Some(receipt_bytes) = residue.receipt.as_deref() else {
        let Some(stage) = residue.receipt_stage.as_deref() else {
            return Ok(Plan::Resume {
                resume: Phase::SynchronizeRootAfterMarkerCleanup,
            });
        };
        return match classify_stage(Stage::Receipt, stage, MIGRATION_RECEIPT_LENGTH)? {
            StageShape::Incomplete => Ok(Plan::DiscardStage {
                stage: Stage::Receipt,
                resume: Phase::WriteReceiptStage,
            }),
            StageShape::Complete => {
                AdmittedStoreMigrationReceipt::decode(stage, intent, marker)
                    .map(|_| ())
                    .map_err(|source| Ambiguity::StageUndecodable {
                        source: DecodeError::Receipt { source },
                        stage: Stage::Receipt,
                    })?;
                Ok(Plan::Resume {
                    resume: Phase::SynchronizeReceiptStage,
                })
            }
        };
    };
    AdmittedStoreMigrationReceipt::decode(receipt_bytes, intent, marker)
        .map(|_| ())
        .map_err(|source| Ambiguity::ReceiptUndecodable { source })?;
    match residue.receipt_stage.as_deref() {
        None => Ok(Plan::Complete),
        Some(stage) if stage == receipt_bytes => Ok(Plan::Resume {
            resume: Phase::SynchronizeRootAfterReceipt,
        }),
        Some(_) => Err(Ambiguity::StageDiffers {
            stage: Stage::Receipt,
        }),
    }
}

enum StageShape {
    Incomplete,
    Complete,
}

fn classify_stage(stage: Stage, bytes: &[u8], length: usize) -> Result<StageShape, Ambiguity> {
    match bytes.len().cmp(&length) {
        std::cmp::Ordering::Less => Ok(StageShape::Incomplete),
        std::cmp::Ordering::Equal => Ok(StageShape::Complete),
        std::cmp::Ordering::Greater => Err(Ambiguity::StageOverlong {
            stage,
            observed: bytes.len(),
        }),
    }
}

/// Every intent coordinate but the mount identity.
fn restart_stable_match(
    expected: &AdmittedStoreMigrationIntent<'_>,
    observed: &AdmittedStoreMigrationIntent<'_>,
) -> bool {
    expected.catalog_generation() == observed.catalog_generation()
        && expected.catalog_length() == observed.catalog_length()
        && expected.catalog_digest() == observed.catalog_digest()
        && expected.predecessor_catalog_digest() == observed.predecessor_catalog_digest()
        && expected.inventory_digest() == observed.inventory_digest()
        && expected.root_device_identity() == observed.root_device_identity()
        && expected.root_file_identity() == observed.root_file_identity()
        && expected.target_definition_digest() == observed.target_definition_digest()
        && expected.store_identifier() == observed.store_identifier()
}
