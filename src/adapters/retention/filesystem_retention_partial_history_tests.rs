//! This module owns initial-record predecessor-prefix refusals and successor completion laws.

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation,
    open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionHeadDecodeError,
    RetentionManifestDecodeError, RetentionRecoveryRefusal, RetentionRootDecodeError,
    RetentionStageAssessment, assess_head_stage, assess_manifest_stage, assess_root_stage,
};
use std::{error::Error, fs};

struct Case {
    stage: Stage,
    name: &'static str,
    corpus: &'static str,
    phases: usize,
    generation: usize,
    predecessor: usize,
}
const CASES: [Case; 3] = [
    Case {
        stage: Stage::Root,
        name: "root.next",
        corpus: ROOT_HEX,
        phases: 1,
        generation: 32,
        predecessor: 116,
    },
    Case {
        stage: Stage::Manifest,
        name: "manifest.next",
        corpus: MANIFEST_HEX,
        phases: 7,
        generation: 32,
        predecessor: 48,
    },
    Case {
        stage: Stage::Head,
        name: "head.next",
        corpus: HEAD_HEX,
        phases: 11,
        generation: 24,
        predecessor: 72,
    },
];

// Size: medium. Oracle: initial records have no predecessor, encoded as all-zero digest bytes.
// Delete only when stronger public recovery laws subsume all record types and partial endpoints.
#[test]
fn nonzero_partial_initial_predecessors_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for case in CASES {
        let bytes = fixture(case.corpus)?;
        for available in 1..32 {
            let end = case
                .predecessor
                .checked_add(available)
                .ok_or("prefix overflow")?;
            let offset = end.checked_sub(1).ok_or("empty prefix")?;
            let mut partial = bytes.get(..end).ok_or("missing prefix")?.to_vec();
            *partial.get_mut(offset).ok_or("missing predecessor byte")? = 7;
            let (sandbox, mut authority) =
                open_authority(&format!("partial-history-{}-{available}", case.name))?;
            let root = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root)?;
            drive_publication(&mut authority, &preparation, case.phases)?;
            fs::write(sandbox.path().join("retention").join(case.name), partial)?;
            let before = retention_witness(sandbox.path())?;
            match authority.recover() {
                Err(FilesystemRetentionRecoveryError::Plan {
                    source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
                }) => {
                    assert_eq!(
                        stage, case.stage,
                        "identify the contradictory partial-history stage"
                    );
                    require_cause(source.as_ref(), case.stage, offset);
                }
                result => {
                    return Err(format!(
                        "{} initial predecessor prefix {available} must refuse: {result:?}",
                        case.name
                    )
                    .into());
                }
            }
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "partial-history refusal must preserve retained evidence"
            );
        }
    }
    Ok(())
}

// Size: small. Oracle: any partial successor predecessor, including zeros, can complete nonzero.
// Delete only when stronger public assessment laws subsume successor completion at all endpoints.
#[test]
fn successor_predecessors_remain_possible_before_the_digest_is_complete()
-> Result<(), Box<dyn Error>> {
    for case in CASES {
        for (generation, byte) in [(2_u64, 0_u8), (2, 7), (u64::MAX, 0), (u64::MAX, 7)] {
            let mut bytes = fixture(case.corpus)?;
            let generation_end = case
                .generation
                .checked_add(8)
                .ok_or("generation overflow")?;
            bytes
                .get_mut(case.generation..generation_end)
                .ok_or("missing generation")?
                .copy_from_slice(&generation.to_be_bytes());
            let predecessor_end = case
                .predecessor
                .checked_add(32)
                .ok_or("predecessor overflow")?;
            bytes
                .get_mut(case.predecessor..predecessor_end)
                .ok_or("missing predecessor")?
                .fill(byte);
            for available in 1..32 {
                let end = case
                    .predecessor
                    .checked_add(available)
                    .ok_or("prefix overflow")?;
                let partial = bytes.get(..end).ok_or("missing prefix")?;
                let possible = match case.stage {
                    Stage::Root => matches!(
                        assess_root_stage(Some(partial)),
                        RetentionStageAssessment::Truncated { .. }
                    ),
                    Stage::Manifest => matches!(
                        assess_manifest_stage(Some(partial)),
                        RetentionStageAssessment::Truncated { .. }
                    ),
                    Stage::Head => matches!(
                        assess_head_stage(Some(partial)),
                        RetentionStageAssessment::Truncated { .. }
                    ),
                };
                assert!(
                    possible,
                    "{} successor generation {generation}, predecessor prefix {available}, must admit nonzero completion",
                    case.name
                );
            }
        }
    }
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), stage: Stage, offset: usize) {
    let actual = match stage {
        Stage::Root => match source.downcast_ref::<RetentionRootDecodeError>() {
            Some(RetentionRootDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }) => Some((*offset, *expected, *observed)),
            _ => None,
        },
        Stage::Manifest => match source.downcast_ref::<RetentionManifestDecodeError>() {
            Some(RetentionManifestDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }) => Some((*offset, *expected, *observed)),
            _ => None,
        },
        Stage::Head => match source.downcast_ref::<RetentionHeadDecodeError>() {
            Some(RetentionHeadDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }) => Some((*offset, *expected, *observed)),
            _ => None,
        },
    };
    assert_eq!(
        actual,
        Some((offset, 0, 7)),
        "report only the exact available predecessor contradiction: {source:?}"
    );
}
