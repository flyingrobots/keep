//! This module owns preservation of semantically invalid interrupted head fields.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionHeadDecodeError,
    RetentionRecoveryRefusal,
};
use crate::RetentionManifestLengthError;

// Size: medium. Oracle: the format's length bounds and entry alignment apply to available fields.
// Delete only if stronger public recovery evidence preserves every invalid-length class here.
#[test]
fn invalid_manifest_lengths_in_short_heads_preserve_corrupt_evidence() -> Result<(), Box<dyn Error>>
{
    for value in [0, 223, 225, 295_137, u64::MAX] {
        let expected = if value == 225 {
            RetentionManifestLengthError::NotCongruent { observed: value }
        } else {
            RetentionManifestLengthError::OutOfBounds {
                minimum: 224,
                maximum: 295_136,
                observed: value,
            }
        };
        let (sandbox, mut authority) = open_authority(&format!("short-head-length-{value}"))?;
        let root = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root)?;
        drive_publication(&mut authority, &preparation, 11)?;
        let publication = preparation.publication().ok_or("missing publication")?;
        let mut partial = publication
            .head()
            .encoded()
            .get(..40)
            .ok_or("short head fixture")?
            .to_vec();
        partial
            .get_mut(32..40)
            .ok_or("missing length field")?
            .copy_from_slice(&value.to_be_bytes());
        fs::write(sandbox.path().join("retention/head.next"), partial)?;
        let before = retention_witness(sandbox.path())?;

        let result = authority.recover();

        assert!(
            matches!(&result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::StageCorrupt { stage: RetentionFixedStage::Head, source },
            }) if matches!(source.downcast_ref::<RetentionHeadDecodeError>(),
                Some(RetentionHeadDecodeError::ManifestLength { source }) if *source == expected)),
            "short head length {value} must refuse with {expected:?}: {result:?}"
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "invalid short head length {value} must preserve retained evidence"
        );
    }
    Ok(())
}

// Size: medium. Oracle: initial heads have no predecessor; successors require one.
// Delete only when stronger public recovery evidence subsumes both history contradictions.
#[test]
fn contradictory_history_in_short_heads_preserves_evidence() -> Result<(), Box<dyn Error>> {
    use crate::{LivenessGeneration, RetentionHeadError, RetentionManifestDigest};
    for (generation, predecessor, expected) in [
        (
            1_u64,
            [7_u8; 32],
            RetentionHeadError::InitialGenerationHasPredecessor {
                observed: RetentionManifestDigest::from_hash([7; 32]),
            },
        ),
        (
            2,
            [0; 32],
            RetentionHeadError::MissingPredecessor {
                generation: LivenessGeneration::new(2)?,
            },
        ),
    ] {
        let (sandbox, mut authority) = open_authority(&format!("short-head-history-{generation}"))?;
        let root = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root)?;
        drive_publication(&mut authority, &preparation, 11)?;
        let publication = preparation.publication().ok_or("missing publication")?;
        let mut partial = publication
            .head()
            .encoded()
            .get(..104)
            .ok_or("short head fixture")?
            .to_vec();
        partial
            .get_mut(24..32)
            .ok_or("missing generation")?
            .copy_from_slice(&generation.to_be_bytes());
        partial
            .get_mut(72..104)
            .ok_or("missing predecessor")?
            .copy_from_slice(&predecessor);
        fs::write(sandbox.path().join("retention/head.next"), partial)?;
        let before = retention_witness(sandbox.path())?;

        let result = authority.recover();

        assert!(
            matches!(&result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::StageCorrupt { stage: RetentionFixedStage::Head, source },
            }) if matches!(source.downcast_ref::<RetentionHeadDecodeError>(),
                Some(RetentionHeadDecodeError::Semantic { source }) if *source == expected)),
            "short head generation {generation} must refuse with {expected:?}: {result:?}"
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "contradictory short head generation {generation} must preserve retained evidence"
        );
    }
    Ok(())
}

// Size: medium. Oracle: every available checksum byte must match the golden head checksum.
// Delete only when stronger public recovery coverage subsumes interrupted-checksum corruption.
#[test]
fn contradictory_partial_head_checksums_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for offset in 112_usize..143 {
        let (sandbox, mut authority) = open_authority(&format!("short-head-checksum-{offset}"))?;
        let root = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root)?;
        drive_publication(&mut authority, &preparation, 11)?;
        let publication = preparation.publication().ok_or("missing publication")?;
        let end = offset.checked_add(1).ok_or("checksum prefix overflow")?;
        let mut partial = publication
            .head()
            .encoded()
            .get(..end)
            .ok_or("missing checksum prefix")?
            .to_vec();
        let byte = partial.get_mut(offset).ok_or("missing checksum byte")?;
        let expected = *byte;
        *byte ^= 1;
        let observed = *byte;
        fs::write(sandbox.path().join("retention/head.next"), partial)?;
        let before = retention_witness(sandbox.path())?;

        let result = authority.recover();

        assert!(
            matches!(&result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::StageCorrupt { stage: RetentionFixedStage::Head, source },
            }) if source.downcast_ref::<RetentionHeadDecodeError>() ==
                Some(&RetentionHeadDecodeError::PrefixByteMismatch { offset, expected, observed })),
            "checksum prefix ending at {end} must report the exact contradictory byte: {result:?}"
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "corrupt checksum prefix ending at {end} must preserve retained evidence"
        );
    }
    Ok(())
}
