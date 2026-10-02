//! This module owns runtime refusal of incomplete, impossible retention size fields.

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

type StagePrefix = (Stage, Vec<u8>);

// Size: medium. Oracle: canonical framing has bounded counts, namespace lengths and entry widths.
// Delete only when stronger runtime laws subsume impossible-prefix refusal and evidence preservation.
#[test]
fn impossible_framing_prefixes_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (index, (stage, bytes)) in impossible_prefixes()?.into_iter().enumerate() {
        let (name, phases) = match stage {
            Stage::Root => ("root.next", 1),
            Stage::Manifest => ("manifest.next", 7),
            Stage::Head => ("head.next", 11),
        };
        let (sandbox, mut authority) = open_authority(&format!("framing-prefix-{index}"))?;
        let root = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root)?;
        drive_publication(&mut authority, &preparation, phases)?;
        fs::write(sandbox.path().join("retention").join(name), &bytes)?;
        let before = retention_witness(sandbox.path())?;
        match authority.recover() {
            Err(FilesystemRetentionRecoveryError::Plan {
                source:
                    RetentionRecoveryRefusal::StageCorrupt {
                        stage: actual,
                        source,
                    },
            }) => {
                assert_eq!(actual, stage, "identify the impossible framing stage");
                require_cause(source.as_ref(), stage, bytes.len());
            }
            result => {
                return Err(
                    format!("{name} impossible prefix {index} must refuse: {result:?}").into(),
                );
            }
        }
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "framing refusal must preserve retained evidence"
        );
    }
    Ok(())
}

fn impossible_prefixes() -> Result<Vec<StagePrefix>, Box<dyn Error>> {
    let mut cases = Vec::new();
    // A nonzero high length byte exceeds every format ceiling at each partial endpoint.
    for (stage, corpus, offset) in [
        (Stage::Root, ROOT_HEX, 24_usize),
        (Stage::Manifest, MANIFEST_HEX, 24),
        (Stage::Head, HEAD_HEX, 32),
    ] {
        for available in 1..8 {
            let end = offset.checked_add(available).ok_or("prefix overflow")?;
            let mut bytes = fixture(corpus)?;
            *bytes.get_mut(offset).ok_or("length byte absent")? = 1;
            bytes.truncate(end);
            cases.push((stage, bytes));
        }
    }
    for (stage, corpus, declared) in [
        (Stage::Root, ROOT_HEX, 256_u64),
        (Stage::Root, ROOT_HEX, 7_799_296),
        (Stage::Manifest, MANIFEST_HEX, 223),
        (Stage::Manifest, MANIFEST_HEX, 225),
        (Stage::Manifest, MANIFEST_HEX, 295_137),
    ] {
        let mut bytes = fixture(corpus)?;
        bytes
            .get_mut(24..32)
            .ok_or("length absent")?
            .copy_from_slice(&declared.to_be_bytes());
        bytes.truncate(32);
        cases.push((stage, bytes));
    }
    let mut namespace = fixture(ROOT_HEX)?;
    *namespace.get_mut(40).ok_or("namespace absent")? = 1;
    namespace.truncate(41);
    cases.push((Stage::Root, namespace));
    // The declared length requires one item; any available high count byte excludes it.
    for (stage, corpus) in [(Stage::Root, ROOT_HEX), (Stage::Manifest, MANIFEST_HEX)] {
        for end in 45_usize..48 {
            let mut bytes = fixture(corpus)?;
            let offset = end.checked_sub(1).ok_or("empty count")?;
            *bytes.get_mut(offset).ok_or("count absent")? = 1;
            bytes.truncate(end);
            cases.push((stage, bytes));
        }
    }
    let mut namespace = fixture(ROOT_HEX)?;
    namespace
        .get_mut(40..42)
        .ok_or("namespace absent")?
        .copy_from_slice(&4_u16.to_be_bytes());
    namespace.truncate(42);
    cases.push((Stage::Root, namespace));
    cases.extend(insufficient_count_prefixes()?);
    let mut head = fixture(HEAD_HEX)?;
    head.get_mut(32..40)
        .ok_or("length absent")?
        .copy_from_slice(&295_168_u64.to_be_bytes());
    head.truncate(39);
    cases.push((Stage::Head, head));
    Ok(cases)
}

fn insufficient_count_prefixes() -> Result<Vec<StagePrefix>, Box<dyn Error>> {
    let mut cases = Vec::new();
    // A zero three-byte count prefix admits at most 255 items, below the required 256.
    for (stage, corpus, total) in [
        (Stage::Root, ROOT_HEX, 30_723_u64),
        (Stage::Manifest, MANIFEST_HEX, 18_656),
    ] {
        let mut bytes = fixture(corpus)?;
        bytes
            .get_mut(24..32)
            .ok_or("length absent")?
            .copy_from_slice(&total.to_be_bytes());
        bytes.truncate(47);
        cases.push((stage, bytes));
    }
    Ok(cases)
}

// Size: small. Oracle: every generated legal framing tuple supplies a completion for its prefixes.
// Delete only when stronger public assessment sweeps subsume legal counts and namespace boundaries.
#[test]
fn canonical_root_framing_remains_possible_at_every_header_prefix() -> Result<(), Box<dyn Error>> {
    let mut root = fixture(ROOT_HEX)?;
    for count in 0_u32..=65_536 {
        for namespace in [1_u16, 3, 118, 119, 120, 255] {
            let total = 256_u64
                .checked_add(u64::from(namespace))
                .and_then(|n| n.checked_add(u64::from(count).checked_mul(119)?))
                .ok_or("root length overflow")?;
            root.get_mut(24..32)
                .ok_or("length absent")?
                .copy_from_slice(&total.to_be_bytes());
            root.get_mut(40..42)
                .ok_or("namespace absent")?
                .copy_from_slice(&namespace.to_be_bytes());
            root.get_mut(44..48)
                .ok_or("count absent")?
                .copy_from_slice(&count.to_be_bytes());
            for end in 24..48 {
                assert!(
                    matches!(
                        assess_root_stage(root.get(..end)),
                        RetentionStageAssessment::Truncated { .. }
                    ),
                    "legal root framing count {count}, namespace {namespace}, prefix {end} must remain possible"
                );
            }
        }
    }
    Ok(())
}

// Size: small. Oracle: the complete legal manifest-length domain supplies canonical completions.
// Delete only when stronger public assessment laws subsume manifest and head length feasibility.
#[test]
fn canonical_manifest_lengths_remain_possible_at_every_header_prefix() -> Result<(), Box<dyn Error>>
{
    let mut manifest = fixture(MANIFEST_HEX)?;
    let mut head = fixture(HEAD_HEX)?;
    for count in 0_u32..=4_096 {
        let total = 224_u64
            .checked_add(
                u64::from(count)
                    .checked_mul(72)
                    .ok_or("entry length overflow")?,
            )
            .ok_or("manifest length overflow")?;
        manifest
            .get_mut(24..32)
            .ok_or("length absent")?
            .copy_from_slice(&total.to_be_bytes());
        manifest
            .get_mut(44..48)
            .ok_or("count absent")?
            .copy_from_slice(&count.to_be_bytes());
        head.get_mut(32..40)
            .ok_or("length absent")?
            .copy_from_slice(&total.to_be_bytes());
        for end in 24..48 {
            assert!(
                matches!(
                    assess_manifest_stage(manifest.get(..end)),
                    RetentionStageAssessment::Truncated { .. }
                ),
                "legal manifest count {count}, prefix {end} must remain possible"
            );
        }
        for end in 32..40 {
            assert!(
                matches!(
                    assess_head_stage(head.get(..end)),
                    RetentionStageAssessment::Truncated { .. }
                ),
                "legal head length {total}, prefix {end} must remain possible"
            );
        }
    }
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), stage: Stage, length: usize) {
    let observed = match stage {
        Stage::Root => match source.downcast_ref::<RetentionRootDecodeError>() {
            Some(RetentionRootDecodeError::FramingPrefixImpossible { observed }) => Some(*observed),
            _ => None,
        },
        Stage::Manifest => match source.downcast_ref::<RetentionManifestDecodeError>() {
            Some(RetentionManifestDecodeError::FramingPrefixImpossible { observed }) => {
                Some(*observed)
            }
            _ => None,
        },
        Stage::Head => match source.downcast_ref::<RetentionHeadDecodeError>() {
            Some(RetentionHeadDecodeError::ManifestLengthPrefixImpossible { observed }) => {
                Some(*observed)
            }
            _ => None,
        },
    };
    assert_eq!(
        observed,
        Some(length),
        "report impossible framing at the actual interrupted length: {source:?}"
    );
}
