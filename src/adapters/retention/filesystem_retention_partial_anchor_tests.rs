//! This module owns preservation of contradictory coordinates inside partial root anchors.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError,
};
use crate::LayoutIdBinaryParseError;
use std::{error::Error, fs};

#[derive(Debug)]
enum Fault {
    Byte {
        offset: usize,
        expected: u8,
        observed: u8,
    },
    Length {
        index: u32,
        source: LayoutIdBinaryParseError,
    },
    PrefixLength {
        index: u32,
        minimum: u64,
        maximum: u64,
    },
}

// Size: medium. Oracle: canonical embedded BlobId/LayoutId fixed bytes from the conformance root.
// Delete only when stronger recovery evidence subsumes each fixed field at its first available byte.
#[test]
fn contradictory_partial_anchor_fields_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for index in [0_u32, 1] {
        for relative in (0_usize..19).chain(59..79) {
            let length = relative.checked_add(1).ok_or("length overflow")?;
            let PartialAnchor { mut prefix, start } = partial_anchor(index, length)?;
            let offset = start.checked_add(relative).ok_or("offset overflow")?;
            let expected = *prefix.get(offset).ok_or("missing field byte")?;
            let observed = expected ^ 1;
            *prefix.get_mut(offset).ok_or("missing mutation byte")? = observed;
            require_preservation(
                &prefix,
                &Fault::Byte {
                    offset,
                    expected,
                    observed,
                },
            )?;
        }
    }
    Ok(())
}

// Size: medium. Oracle: v1 layout lengths lie in 176..=46137520 and equal 176 modulo 44.
// Delete only when stronger recovery laws subsume early complete-length admission at both indices.
#[test]
fn invalid_lengths_in_partial_anchors_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for index in [0_u32, 1] {
        for length in [87, 118] {
            for observed in [0_u64, 177, 46_137_521] {
                let PartialAnchor { mut prefix, start } = partial_anchor(index, length)?;
                let field_start = start.checked_add(79).ok_or("offset overflow")?;
                let field_end = start.checked_add(87).ok_or("offset overflow")?;
                prefix
                    .get_mut(field_start..field_end)
                    .ok_or("missing plan length")?
                    .copy_from_slice(&observed.to_be_bytes());
                let source = if observed == 177 {
                    LayoutIdBinaryParseError::PlanLengthNotCongruent { observed }
                } else {
                    LayoutIdBinaryParseError::PlanLengthOutOfBounds {
                        minimum: 176,
                        maximum: 46_137_520,
                        observed,
                    }
                };
                require_preservation(&prefix, &Fault::Length { index, source })?;
            }
        }
    }
    Ok(())
}

struct PartialAnchor {
    prefix: Vec<u8>,
    start: usize,
}

fn partial_anchor(index: u32, length: usize) -> Result<PartialAnchor, Box<dyn Error>> {
    let root = fixture(ROOT_HEX)?;
    let mut prefix = root
        .get(..195)
        .ok_or("missing root header and namespace")?
        .to_vec();
    let anchor = root.get(195..314).ok_or("missing conformance anchor")?;
    for _ in 0..index {
        prefix.extend_from_slice(anchor);
    }
    let start = prefix.len();
    prefix.extend_from_slice(anchor.get(..length).ok_or("invalid anchor prefix length")?);
    let count = index.checked_add(1).ok_or("count overflow")?;
    let total = u64::from(count)
        .checked_mul(119)
        .and_then(|size| size.checked_add(259))
        .ok_or("record length overflow")?;
    prefix
        .get_mut(24..32)
        .ok_or("missing record length")?
        .copy_from_slice(&total.to_be_bytes());
    prefix
        .get_mut(44..48)
        .ok_or("missing count")?
        .copy_from_slice(&count.to_be_bytes());
    Ok(PartialAnchor { prefix, start })
}

fn require_preservation(prefix: &[u8], expected: &Fault) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(&format!("partial-anchor-{}", prefix.len()))?;
    fs::write(sandbox.path().join("retention/root.next"), prefix)?;
    let before = retention_witness(sandbox.path())?;
    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
        }) => {
            assert_eq!(
                stage,
                RetentionFixedStage::Root,
                "identify the corrupt partial-anchor stage"
            );
            require_cause(source.as_ref(), expected);
        }
        result => {
            return Err(format!("partial anchor must refuse {expected:?}: {result:?}").into());
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "partial-anchor refusal must preserve retained evidence"
    );
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), expected: &Fault) {
    let exact = match (source.downcast_ref::<RetentionRootDecodeError>(), expected) {
        (
            Some(RetentionRootDecodeError::LayoutLengthPrefixAboveMaximum {
                index,
                minimum,
                maximum,
            }),
            Fault::PrefixLength {
                index: wanted_index,
                minimum: wanted_minimum,
                maximum: wanted_maximum,
            },
        ) => index == wanted_index && minimum == wanted_minimum && maximum == wanted_maximum,
        (
            Some(RetentionRootDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }),
            Fault::Byte {
                offset: wanted_offset,
                expected: wanted_expected,
                observed: wanted_observed,
            },
        ) => offset == wanted_offset && expected == wanted_expected && observed == wanted_observed,
        (
            Some(RetentionRootDecodeError::LayoutId { index, source }),
            Fault::Length {
                index: wanted_index,
                source: wanted_source,
            },
        ) => index == wanted_index && source == wanted_source,
        _ => false,
    };
    assert!(
        exact,
        "report exact partial-anchor cause: expected {expected:?}, observed {source:?}"
    );
}

// Size: medium. Oracle: every completion of these big-endian prefixes exceeds 46137520.
// Delete only when stronger recovery laws subsume impossible partial numeric lengths.
#[test]
fn impossible_partial_layout_lengths_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for index in [0_u32, 1] {
        for available in 1_usize..8 {
            require_length_prefix(index, available, 0xff00_0000_0000_0000)?;
        }
        require_length_prefix(index, 7, 46_137_600)?;
    }
    Ok(())
}

fn require_length_prefix(index: u32, available: usize, minimum: u64) -> Result<(), Box<dyn Error>> {
    let length = 79_usize.checked_add(available).ok_or("length overflow")?;
    let PartialAnchor { mut prefix, start } = partial_anchor(index, length)?;
    let field_start = start.checked_add(79).ok_or("offset overflow")?;
    let field_end = field_start
        .checked_add(available)
        .ok_or("offset overflow")?;
    prefix
        .get_mut(field_start..field_end)
        .ok_or("missing partial length")?
        .copy_from_slice(
            minimum
                .to_be_bytes()
                .get(..available)
                .ok_or("missing source bytes")?,
        );
    require_preservation(
        &prefix,
        &Fault::PrefixLength {
            index,
            minimum,
            maximum: 46_137_520,
        },
    )
}

// Size: small. Oracle: a canonical layout value witnesses a valid completion of every prefix.
// Delete only when stronger public assessment evidence subsumes these byte-boundary controls.
#[test]
fn canonical_layout_lengths_admit_every_partial_numeric_prefix() -> Result<(), Box<dyn Error>> {
    for count in std::iter::once(0_u64).chain((0..=20).filter_map(|bit| 1_u64.checked_shl(bit))) {
        let length = count
            .checked_mul(44)
            .and_then(|size| size.checked_add(176))
            .ok_or("layout length overflow")?;
        for available in 1_usize..8 {
            let partial_length = 79_usize
                .checked_add(available)
                .ok_or("prefix length overflow")?;
            let PartialAnchor { mut prefix, start } = partial_anchor(0, partial_length)?;
            let field_start = start.checked_add(79).ok_or("offset overflow")?;
            let field_end = field_start
                .checked_add(available)
                .ok_or("offset overflow")?;
            prefix
                .get_mut(field_start..field_end)
                .ok_or("missing target bytes")?
                .copy_from_slice(
                    length
                        .to_be_bytes()
                        .get(..available)
                        .ok_or("missing source bytes")?,
                );
            let assessment = super::assess_root_stage(Some(&prefix));
            assert!(
                matches!(
                    assessment,
                    super::RetentionStageAssessment::Truncated { .. }
                ),
                "valid layout length {length} with {available} available bytes must admit completion: {assessment:?}"
            );
        }
    }
    Ok(())
}
