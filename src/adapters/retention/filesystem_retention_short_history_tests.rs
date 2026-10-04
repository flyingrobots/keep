//! This module owns preservation of contradictory interrupted root and manifest histories.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionRootDecodeError,
};
use crate::{RetentionManifestError, RetentionRootError};
use std::{error::Error, fs};

// Size: medium. Oracle: initial records omit predecessors; successor records require them.
// Delete only when stronger public recovery coverage subsumes both record/history combinations.
#[test]
fn contradictory_short_root_and_manifest_histories_preserve_evidence() -> Result<(), Box<dyn Error>>
{
    for (stage, name, corpus, phases, start, end) in [
        (Stage::Root, "root.next", ROOT_HEX, 1, 116, 148),
        (Stage::Manifest, "manifest.next", MANIFEST_HEX, 7, 48, 80),
    ] {
        for (generation, predecessor) in [(1_u64, [7_u8; 32]), (2, [0; 32])] {
            let (sandbox, mut authority) =
                open_authority(&format!("short-history-{name}-{generation}"))?;
            let root = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root)?;
            drive_publication(&mut authority, &preparation, phases)?;
            let bytes = fixture(corpus)?;
            let mut partial = bytes.get(..end).ok_or("short fixture header")?.to_vec();
            partial
                .get_mut(32..40)
                .ok_or("missing generation")?
                .copy_from_slice(&generation.to_be_bytes());
            partial
                .get_mut(start..end)
                .ok_or("missing predecessor")?
                .copy_from_slice(&predecessor);
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
                    assert_eq!(actual, stage, "name the contradictory history stage");
                    require_history_cause(source.as_ref(), stage, generation);
                }
                result => return Err(format!(
                    "{name} generation {generation} must refuse contradictory history: {result:?}"
                )
                .into()),
            }
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "{name} generation {generation} must preserve retained evidence"
            );
        }
    }
    Ok(())
}

fn require_history_cause(source: &(dyn Error + 'static), stage: Stage, generation: u64) {
    let exact = match stage {
        Stage::Root => match source.downcast_ref::<RetentionRootDecodeError>() {
            Some(RetentionRootDecodeError::Semantic {
                source: RetentionRootError::InitialGenerationHasPredecessor { observed },
            }) => generation == 1 && observed.as_bytes() == &[7; 32],
            Some(RetentionRootDecodeError::Semantic {
                source:
                    RetentionRootError::MissingPredecessor {
                        generation: observed,
                    },
            }) => generation == 2 && observed.get() == generation,
            _ => false,
        },
        _ => match source.downcast_ref::<RetentionManifestDecodeError>() {
            Some(RetentionManifestDecodeError::Semantic {
                source: RetentionManifestError::InitialGenerationHasPredecessor { observed },
            }) => generation == 1 && observed.as_bytes() == &[7; 32],
            Some(RetentionManifestDecodeError::Semantic {
                source:
                    RetentionManifestError::MissingPredecessor {
                        generation: observed,
                    },
            }) => generation == 2 && observed.get() == generation,
            _ => false,
        },
    };
    assert!(
        exact,
        "{stage:?} generation {generation} must report its exact history contradiction: {source:?}"
    );
}
