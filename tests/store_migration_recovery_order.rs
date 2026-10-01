//! This module owns refusal of migration stages surviving later effects.

mod support;

use keep::{
    AdmittedStoreMigrationIntent, StoreMigrationEffect, StoreMigrationFixedStage,
    StoreMigrationRecoveryAmbiguity, StoreMigrationResidue, plan_store_migration_recovery,
};
use std::error::Error;

#[test]
fn an_intent_stage_cannot_survive_reader_fence_creation() -> Result<(), Box<dyn Error>> {
    let intent = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-intent.hex").trim(),
    )?;
    let expected = AdmittedStoreMigrationIntent::decode(&intent)?;
    let residue = StoreMigrationResidue {
        intent: Some(intent.clone()),
        intent_stage: Some(intent.clone()),
        reader_fence: true,
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &residue),
        Err(StoreMigrationRecoveryAmbiguity::StageAfterEffect {
            stage: StoreMigrationFixedStage::Intent,
            effect: StoreMigrationEffect::Namespace
        })
    ));
    Ok(())
}

#[test]
fn a_marker_stage_cannot_survive_receipt_publication() -> Result<(), Box<dyn Error>> {
    let intent = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-intent.hex").trim(),
    )?;
    let marker = support::decode_hex(
        include_str!("../conformance/segment-store/v2/format-marker.hex").trim(),
    )?;
    let receipt = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-receipt.hex").trim(),
    )?;
    let expected = AdmittedStoreMigrationIntent::decode(&intent)?;
    let residue = StoreMigrationResidue {
        intent: Some(intent.clone()),
        reader_fence: true,
        namespace_prefix: [true; 6],
        marker: Some(marker.clone()),
        marker_stage: Some(marker),
        receipt: Some(receipt),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &residue),
        Err(StoreMigrationRecoveryAmbiguity::StageAfterEffect {
            stage: StoreMigrationFixedStage::Marker,
            effect: StoreMigrationEffect::Receipt
        })
    ));
    Ok(())
}

#[test]
fn a_receipt_only_later_effect_is_reported_as_receipt() -> Result<(), Box<dyn Error>> {
    let intent = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-intent.hex").trim(),
    )?;
    let receipt = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-receipt.hex").trim(),
    )?;
    let expected = AdmittedStoreMigrationIntent::decode(&intent)?;
    let residue = StoreMigrationResidue {
        intent: Some(intent.clone()),
        intent_stage: Some(intent.clone()),
        receipt: Some(receipt),
        ..StoreMigrationResidue::VERSION_ONE
    };
    assert!(matches!(
        plan_store_migration_recovery(&expected, &residue),
        Err(StoreMigrationRecoveryAmbiguity::StageAfterEffect {
            stage: StoreMigrationFixedStage::Intent,
            effect: StoreMigrationEffect::Receipt,
        })
    ));
    Ok(())
}

#[test]
fn a_receipt_before_the_marker_reports_the_missing_marker_at_every_partial_namespace()
-> Result<(), Box<dyn Error>> {
    let intent = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-intent.hex").trim(),
    )?;
    let receipt = support::decode_hex(
        include_str!("../conformance/segment-store/v2/migration-receipt.hex").trim(),
    )?;
    let expected = AdmittedStoreMigrationIntent::decode(&intent)?;
    for extent in 0..7 {
        for (receipt_stage, canonical) in
            [(Some(receipt.clone()), None), (None, Some(receipt.clone()))]
        {
            let residue = StoreMigrationResidue {
                intent: Some(intent.clone()),
                reader_fence: extent > 0,
                namespace_prefix: std::array::from_fn(|position: usize| {
                    position.checked_add(1).is_some_and(|next| next < extent)
                }),
                receipt_stage,
                receipt: canonical,
                ..StoreMigrationResidue::VERSION_ONE
            };
            assert!(matches!(
                plan_store_migration_recovery(&expected, &residue),
                Err(StoreMigrationRecoveryAmbiguity::ReceiptBeforeMarker)
            ));
        }
    }
    Ok(())
}
