//! Storage-independent migration recovery planning laws.
//!
//! One law per row of the recovery table in `migration-recovery.md`, plus
//! one per ambiguity rule, each over the frozen version-2 records.

mod support;

use std::error::Error;

use keep::{
    AdmittedStoreMigrationIntent, StoreMigrationEffect, StoreMigrationFixedStage as Stage,
    StoreMigrationPhase as Phase, StoreMigrationRecoveryAmbiguity as Ambiguity,
    StoreMigrationRecoveryPlan as Plan, StoreMigrationResidue, plan_store_migration_recovery,
};

use crate::support::{domain_hash, patch};

const INTENT: &str = include_str!("../conformance/segment-store/v2/migration-intent.hex");
const MARKER: &str = include_str!("../conformance/segment-store/v2/format-marker.hex");
const RECEIPT: &str = include_str!("../conformance/segment-store/v2/migration-receipt.hex");
const INTENT_CHECKSUM_OFFSET: usize = 224;
const ROOT_DEVICE_OFFSET: usize = 136;
const ROOT_MOUNT_OFFSET: usize = 144;

struct Records {
    intent: Vec<u8>,
    marker: Vec<u8>,
    receipt: Vec<u8>,
}

#[test]
fn no_artifact_admits_version_one() -> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;
    assert_eq!(
        plan_store_migration_recovery(&expected, &StoreMigrationResidue::VERSION_ONE)?,
        Plan::VersionOne
    );
    Ok(())
}

#[test]
fn an_intent_stage_alone_resumes_when_exact_and_is_discarded_when_incomplete()
-> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;

    let exact = StoreMigrationResidue {
        intent_stage: Some(records.intent.clone()),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &exact)?,
        Plan::Resume {
            resume: Phase::SynchronizeIntentStage
        }
    );

    let mut truncated = records.intent.clone();
    truncated.truncate(100);
    let incomplete = StoreMigrationResidue {
        intent_stage: Some(truncated),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &incomplete)?,
        Plan::DiscardStage {
            stage: Stage::Intent,
            resume: Phase::WriteIntentStage,
        }
    );

    let mut overlong = records.intent.clone();
    overlong.push(0);
    let overlong = StoreMigrationResidue {
        intent_stage: Some(overlong),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &overlong),
        Err(Ambiguity::StageOverlong {
            stage: Stage::Intent,
            observed: 257,
        })
    ));

    let mut corrupt = records.intent.clone();
    let last = corrupt.last_mut().ok_or("intent is empty")?;
    *last ^= 1;
    let corrupt = StoreMigrationResidue {
        intent_stage: Some(corrupt),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &corrupt),
        Err(Ambiguity::StageUndecodable {
            stage: Stage::Intent
        })
    ));
    Ok(())
}

#[test]
fn a_durable_intent_resumes_after_its_cleanup_and_binds_restart_stable_coordinates()
-> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;

    let durable = StoreMigrationResidue {
        intent: Some(records.intent.clone()),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &durable)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterIntentCleanup
        }
    );

    let with_stage = StoreMigrationResidue {
        intent_stage: Some(records.intent.clone()),
        ..durable.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &with_stage)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterIntent
        }
    );

    let mut remounted = records.intent.clone();
    patch(&mut remounted, ROOT_MOUNT_OFFSET, &9_u64.to_be_bytes())?;
    reseal_intent(&mut remounted)?;
    let remounted = AdmittedStoreMigrationIntent::decode(&remounted)?;
    assert_eq!(
        plan_store_migration_recovery(&remounted, &durable)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterIntentCleanup
        },
        "a rebooted root's new mount id must not reject the store's own intent"
    );

    let mut moved = records.intent.clone();
    patch(&mut moved, ROOT_DEVICE_OFFSET, &9_u64.to_be_bytes())?;
    reseal_intent(&mut moved)?;
    let moved = AdmittedStoreMigrationIntent::decode(&moved)?;
    assert!(matches!(
        plan_store_migration_recovery(&moved, &durable),
        Err(Ambiguity::IntentDiffers)
    ));

    let mut corrupt = durable;
    if let Some(bytes) = corrupt.intent.as_mut()
        && let Some(last) = bytes.last_mut()
    {
        *last ^= 1;
    }
    assert!(matches!(
        plan_store_migration_recovery(&expected, &corrupt),
        Err(Ambiguity::IntentUndecodable { .. })
    ));
    Ok(())
}

#[test]
fn every_effect_needs_a_durable_intent_first() -> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;
    for (residue, effect) in [
        (
            StoreMigrationResidue {
                reader_fence: true,
                ..StoreMigrationResidue::VERSION_ONE
            },
            StoreMigrationEffect::Namespace,
        ),
        (
            StoreMigrationResidue {
                marker: Some(records.marker.clone()),
                ..StoreMigrationResidue::VERSION_ONE
            },
            StoreMigrationEffect::Marker,
        ),
        (
            StoreMigrationResidue {
                receipt_stage: Some(records.receipt.clone()),
                ..StoreMigrationResidue::VERSION_ONE
            },
            StoreMigrationEffect::Receipt,
        ),
    ] {
        assert!(matches!(
            plan_store_migration_recovery(&expected, &residue),
            Err(Ambiguity::EffectBeforeIntent { effect: observed }) if observed == effect
        ));
    }
    Ok(())
}

#[test]
fn the_namespace_prefix_resumes_in_order_and_refuses_gaps() -> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;
    let durable = StoreMigrationResidue {
        intent: Some(records.intent.clone()),
        ..StoreMigrationResidue::VERSION_ONE
    };

    let partial = StoreMigrationResidue {
        reader_fence: true,
        namespace_prefix: [true, true, false, false, false, false],
        ..durable.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &partial)?,
        Plan::Resume {
            resume: Phase::AdmitNamespacePrefix
        }
    );

    let complete = StoreMigrationResidue {
        reader_fence: true,
        namespace_prefix: [true; 6],
        ..durable.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &complete)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterNamespace
        }
    );

    let gap = StoreMigrationResidue {
        reader_fence: true,
        namespace_prefix: [true, false, true, false, false, false],
        ..durable.clone()
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &gap),
        Err(Ambiguity::NamespaceOutOfOrder {
            absent: 2,
            present: 3,
        })
    ));

    let no_fence = StoreMigrationResidue {
        namespace_prefix: [true, false, false, false, false, false],
        ..durable
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &no_fence),
        Err(Ambiguity::NamespaceOutOfOrder {
            absent: 0,
            present: 1,
        })
    ));

    let early_marker = StoreMigrationResidue {
        marker_stage: Some(records.marker.clone()),
        ..partial
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &early_marker),
        Err(Ambiguity::MarkerBeforeNamespace)
    ));
    Ok(())
}

#[test]
fn the_marker_stage_and_marker_resume_at_their_first_unproven_phase() -> Result<(), Box<dyn Error>>
{
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;
    let namespace = StoreMigrationResidue {
        intent: Some(records.intent.clone()),
        reader_fence: true,
        namespace_prefix: [true; 6],
        ..StoreMigrationResidue::VERSION_ONE
    };

    let staged = StoreMigrationResidue {
        marker_stage: Some(records.marker.clone()),
        ..namespace.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &staged)?,
        Plan::Resume {
            resume: Phase::SynchronizeMarkerStage
        }
    );

    let mut short = records.marker.clone();
    short.truncate(10);
    let incomplete = StoreMigrationResidue {
        marker_stage: Some(short),
        ..namespace.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &incomplete)?,
        Plan::DiscardStage {
            stage: Stage::Marker,
            resume: Phase::WriteMarkerStage,
        }
    );

    let linked = StoreMigrationResidue {
        marker: Some(records.marker.clone()),
        marker_stage: Some(records.marker.clone()),
        ..namespace.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &linked)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterMarker
        }
    );

    let published = StoreMigrationResidue {
        marker: Some(records.marker.clone()),
        ..namespace.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &published)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterMarkerCleanup
        }
    );

    let early_receipt = StoreMigrationResidue {
        receipt: Some(records.receipt.clone()),
        ..namespace
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &early_receipt),
        Err(Ambiguity::ReceiptBeforeMarker)
    ));
    Ok(())
}

#[test]
fn the_receipt_completes_the_migration_only_when_exact_and_alone() -> Result<(), Box<dyn Error>> {
    let records = records()?;
    let expected = AdmittedStoreMigrationIntent::decode(&records.intent)?;
    let marked = StoreMigrationResidue {
        intent: Some(records.intent.clone()),
        reader_fence: true,
        namespace_prefix: [true; 6],
        marker: Some(records.marker.clone()),
        ..StoreMigrationResidue::VERSION_ONE
    };

    let staged = StoreMigrationResidue {
        receipt_stage: Some(records.receipt.clone()),
        ..marked.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &staged)?,
        Plan::Resume {
            resume: Phase::SynchronizeReceiptStage
        }
    );

    let linked = StoreMigrationResidue {
        receipt: Some(records.receipt.clone()),
        receipt_stage: Some(records.receipt.clone()),
        ..marked.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &linked)?,
        Plan::Resume {
            resume: Phase::SynchronizeRootAfterReceipt
        }
    );

    let complete = StoreMigrationResidue {
        receipt: Some(records.receipt.clone()),
        ..marked.clone()
    };
    assert_eq!(
        plan_store_migration_recovery(&expected, &complete)?,
        Plan::Complete
    );

    let mut conflicting = records.receipt.clone();
    let last = conflicting.last_mut().ok_or("receipt is empty")?;
    *last ^= 1;
    let conflicting = StoreMigrationResidue {
        receipt: Some(conflicting),
        ..marked
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &conflicting),
        Err(Ambiguity::ReceiptUndecodable { .. })
    ));
    Ok(())
}

fn records() -> Result<Records, Box<dyn Error>> {
    Ok(Records {
        intent: support::decode_hex(INTENT.trim_end())?,
        marker: support::decode_hex(MARKER.trim_end())?,
        receipt: support::decode_hex(RECEIPT.trim_end())?,
    })
}

fn reseal_intent(bytes: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let preimage = bytes
        .get(..INTENT_CHECKSUM_OFFSET)
        .ok_or("intent lacks its checksum preimage")?;
    let checksum = domain_hash(b"keep.store-migration-intent-checksum/v2\0", preimage);
    patch(bytes, INTENT_CHECKSUM_OFFSET, &checksum)?;
    Ok(())
}
