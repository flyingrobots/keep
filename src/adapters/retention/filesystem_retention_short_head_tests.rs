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
