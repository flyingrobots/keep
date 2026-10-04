//! This module owns preservation of excessive counts in interrupted stage headers.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionRootDecodeError,
};
use std::{error::Error, fs};

// Size: medium. Oracle: version-two root/manifest count ceilings apply before stage discard.
// Delete only when stronger public recovery coverage subsumes excessive-count preservation.
#[test]
fn excessive_counts_in_short_headers_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (stage, name, corpus, phases, maximum, width) in [
        (Stage::Root, "root.next", ROOT_HEX, 1, 65_536_u32, 119_u64),
        (Stage::Manifest, "manifest.next", MANIFEST_HEX, 7, 4_096, 72),
    ] {
        for observed in [maximum.checked_add(1).ok_or("count overflow")?, u32::MAX] {
            let (sandbox, mut authority) =
                open_authority(&format!("short-count-{name}-{observed}"))?;
            let root = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root)?;
            drive_publication(&mut authority, &preparation, phases)?;
            let bytes = fixture(corpus)?;
            // Both frozen fixtures contain one entry; keep the declared length consistent.
            let length = u64::from(observed)
                .checked_sub(1)
                .and_then(|n| n.checked_mul(width))
                .and_then(|extra| u64::try_from(bytes.len()).ok()?.checked_add(extra))
                .ok_or("record length overflow")?;
            let mut partial = bytes.get(..48).ok_or("short fixture header")?.to_vec();
            partial
                .get_mut(24..32)
                .ok_or("missing length")?
                .copy_from_slice(&length.to_be_bytes());
            partial
                .get_mut(44..48)
                .ok_or("missing count")?
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
                    assert_eq!(actual, stage, "name the excessive-count stage");
                    require_count_cause(source.as_ref(), stage, maximum, observed);
                }
                result => {
                    return Err(format!("{name} count {observed} must refuse: {result:?}").into());
                }
            }
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "{name} count {observed} must preserve retained evidence"
            );
        }
    }
    Ok(())
}

fn require_count_cause(source: &(dyn Error + 'static), stage: Stage, maximum: u32, observed: u32) {
    let exact = match stage {
        Stage::Root => matches!(source.downcast_ref::<RetentionRootDecodeError>(),
            Some(RetentionRootDecodeError::AnchorCountExceeded { maximum: m, observed: o })
                if *m == maximum && *o == observed),
        _ => matches!(source.downcast_ref::<RetentionManifestDecodeError>(),
            Some(RetentionManifestDecodeError::EntryCountExceeded { maximum: m, observed: o })
                if *m == maximum && *o == observed),
    };
    assert!(
        exact,
        "{stage:?} must report count {observed} above maximum {maximum}: {source:?}"
    );
}
