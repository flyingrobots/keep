//! This module owns preservation of impossible interrupted profile prefixes.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError,
};
use std::{error::Error, fs};

// Size: medium. Oracle: the registered v2 profile has one exact identity/version/digest encoding.
// Delete only when stronger public recovery coverage subsumes every partial profile field.
#[test]
fn contradictory_partial_root_profiles_preserve_evidence() -> Result<(), Box<dyn Error>> {
    let root = fixture(ROOT_HEX)?;
    for end in 49_usize..88 {
        let offset = end.checked_sub(1).ok_or("invalid prefix length")?;
        let expected = *root.get(offset).ok_or("missing profile fixture byte")?;
        let observed = expected ^ 1;
        let mut partial = root.get(..end).ok_or("short fixture")?.to_vec();
        *partial.get_mut(offset).ok_or("missing mutation byte")? = observed;
        let (sandbox, mut authority) = open_authority(&format!("partial-profile-{end}"))?;
        fs::write(sandbox.path().join("retention/root.next"), partial)?;
        let before = retention_witness(sandbox.path())?;

        match authority.recover() {
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
            }) => {
                assert_eq!(
                    stage,
                    RetentionFixedStage::Root,
                    "identify the corrupt profile stage"
                );
                assert!(
                    matches!(source.downcast_ref::<RetentionRootDecodeError>(),
                        Some(RetentionRootDecodeError::PrefixByteMismatch {
                            offset: actual_offset, expected: actual_expected, observed: actual_observed,
                        }) if *actual_offset == offset && *actual_expected == expected && *actual_observed == observed
                    ),
                    "name the exact available profile contradiction at {offset}: {source:?}"
                );
            }
            result => {
                return Err(format!(
                    "profile contradiction at {offset} must refuse recovery: {result:?}"
                )
                .into());
            }
        }
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "profile contradiction at {offset} must preserve retained evidence"
        );
    }
    Ok(())
}
