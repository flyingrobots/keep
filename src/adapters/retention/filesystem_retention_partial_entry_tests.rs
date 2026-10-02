//! This module owns manifest partial-entry refusal and valid-completion laws.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionStageAssessment, assess_manifest_stage,
};
use crate::RootGenerationError;
use std::{error::Error, fs};

// Size: medium. Oracle: manifest namespace digests are strictly increasing.
// Delete only when stronger public recovery laws subsume each impossible partial-order class.
#[test]
fn impossible_partial_namespace_order_preserves_evidence() -> Result<(), Box<dyn Error>> {
    for length in 1..72 {
        require_preservation(&entry_prefix([0x80; 32], [0x7f; 32], length)?, Fault::Order)?;
    }
    for length in 32..72 {
        require_preservation(&entry_prefix([0x80; 32], [0x80; 32], length)?, Fault::Order)?;
    }
    for length in 1..32 {
        require_preservation(&entry_prefix([0xff; 32], [0xff; 32], length)?, Fault::Order)?;
    }
    Ok(())
}

// Size: medium. Oracle: a complete root-generation field must be positive, even before its digest.
// Delete only when stronger public recovery evidence subsumes every remaining entry length.
#[test]
fn zero_generation_in_partial_manifest_entry_preserves_evidence() -> Result<(), Box<dyn Error>> {
    let mut bytes = fixture(MANIFEST_HEX)?;
    bytes
        .get_mut(192..200)
        .ok_or("missing root generation")?
        .fill(0);
    for end in 200..232 {
        require_preservation(
            bytes.get(..end).ok_or("missing partial entry")?,
            Fault::Generation,
        )?;
    }
    Ok(())
}

// Size: small. Oracle: an ordered complete namespace supplies a witness for every prefix.
// Delete only when stronger public assessment coverage subsumes these possible-completion classes.
#[test]
fn ordered_manifest_entries_remain_possible_at_every_partial_length() -> Result<(), Box<dyn Error>>
{
    let mut late_successor = [0x80; 32];
    *late_successor.last_mut().ok_or("empty namespace")? = 0x81;
    for next in [[0x81; 32], late_successor] {
        for length in 1..72 {
            let prefix = entry_prefix([0x80; 32], next, length)?;
            let assessment = assess_manifest_stage(Some(&prefix));
            assert!(
                matches!(assessment, RetentionStageAssessment::Truncated { .. }),
                "ordered namespace prefix length {length} must admit completion: {assessment:?}"
            );
        }
    }
    Ok(())
}

fn entry_prefix(prior: [u8; 32], next: [u8; 32], length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = fixture(MANIFEST_HEX)?;
    let mut prefix = bytes.get(..232).ok_or("missing first entry")?.to_vec();
    prefix
        .get_mut(24..32)
        .ok_or("missing length")?
        .copy_from_slice(&368_u64.to_be_bytes());
    prefix
        .get_mut(44..48)
        .ok_or("missing count")?
        .copy_from_slice(&2_u32.to_be_bytes());
    prefix
        .get_mut(160..192)
        .ok_or("missing prior namespace")?
        .copy_from_slice(&prior);
    let mut entry = bytes.get(160..232).ok_or("missing fixture entry")?.to_vec();
    entry
        .get_mut(..32)
        .ok_or("missing next namespace")?
        .copy_from_slice(&next);
    prefix.extend_from_slice(entry.get(..length).ok_or("invalid entry prefix")?);
    Ok(prefix)
}

#[derive(Clone, Copy, Debug)]
enum Fault {
    Order,
    Generation,
}

fn require_preservation(prefix: &[u8], fault: Fault) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) =
        open_authority(&format!("partial-entry-{fault:?}-{}", prefix.len()))?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 7)?;
    fs::write(sandbox.path().join("retention/manifest.next"), prefix)?;
    let before = retention_witness(sandbox.path())?;
    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
        }) => {
            assert_eq!(
                stage,
                RetentionFixedStage::Manifest,
                "identify the corrupt partial-entry stage"
            );
            let exact = match (fault, source.downcast_ref::<RetentionManifestDecodeError>()) {
                (
                    Fault::Order,
                    Some(RetentionManifestDecodeError::NonCanonicalEntryOrder { index: 1 }),
                ) => true,
                (
                    Fault::Generation,
                    Some(RetentionManifestDecodeError::RootGeneration {
                        index: 0,
                        source: RootGenerationError::Zero,
                    }),
                ) => true,
                _ => false,
            };
            assert!(
                exact,
                "report exact partial-entry {fault:?} cause: {source:?}"
            );
        }
        result => {
            return Err(format!("partial manifest entry {fault:?} must refuse: {result:?}").into());
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "partial-entry refusal must preserve retained evidence"
    );
    Ok(())
}
