//! This module owns preservation of contradictory lengths in interrupted stage headers.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionRootDecodeError,
};
use std::{error::Error, fs};

// Size: medium. Oracle: the declared size equals the format size of the supplied golden record.
// Delete only when stronger public recovery laws preserve contradictory header framing.
#[test]
fn contradictory_lengths_in_short_stage_headers_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (stage, name, corpus, phases) in [
        (Stage::Root, "root.next", ROOT_HEX, 1),
        (Stage::Manifest, "manifest.next", MANIFEST_HEX, 7),
    ] {
        let bytes = fixture(corpus)?;
        let expected = u64::try_from(bytes.len())?;
        for observed in [
            0,
            expected.checked_add(1).ok_or("length overflow")?,
            u64::MAX,
        ] {
            let (sandbox, mut authority) =
                open_authority(&format!("short-framing-{name}-{observed}"))?;
            let root = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root)?;
            drive_publication(&mut authority, &preparation, phases)?;
            let mut partial = bytes.get(..48).ok_or("short fixture header")?.to_vec();
            partial
                .get_mut(24..32)
                .ok_or("missing declared length")?
                .copy_from_slice(&observed.to_be_bytes());
            fs::write(sandbox.path().join("retention").join(name), partial)?;
            let before = retention_witness(sandbox.path())?;

            let result = authority.recover();

            match result {
                Err(FilesystemRetentionRecoveryError::Plan {
                    source:
                        RetentionRecoveryRefusal::StageCorrupt {
                            stage: actual,
                            source,
                        },
                }) => {
                    assert_eq!(actual, stage, "name the contradictory stage");
                    require_length_cause(source.as_ref(), stage, expected, observed);
                }
                result => {
                    return Err(format!(
                        "{name} length {observed} must refuse contradictory framing: {result:?}"
                    )
                    .into());
                }
            }
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "{name} length {observed} must preserve retained evidence"
            );
        }
    }
    Ok(())
}

fn require_length_cause(
    source: &(dyn Error + 'static),
    stage: Stage,
    expected: u64,
    observed: u64,
) {
    let matches_length = match stage {
        Stage::Root => matches!(source.downcast_ref::<RetentionRootDecodeError>(),
            Some(RetentionRootDecodeError::DeclaredLengthMismatch { expected: e, observed: o })
                if *e == expected && *o == observed),
        _ => matches!(source.downcast_ref::<RetentionManifestDecodeError>(),
            Some(RetentionManifestDecodeError::DeclaredLengthMismatch { expected: e, observed: o })
                if *e == expected && *o == observed),
    };
    assert!(
        matches_length,
        "{stage:?} must report expected {expected}, observed {observed}: {source:?}"
    );
}
