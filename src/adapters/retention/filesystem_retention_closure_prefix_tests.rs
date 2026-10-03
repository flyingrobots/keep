//! This module owns refusal and preservation of impossible closure-limit prefixes.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError,
};
use crate::{RetentionClosureLimit as Limit, RetentionClosureLimitError as LimitError};
use std::{error::Error, fs};

// Size: medium. Oracle: v2 closure limits are positive bounded big-endian integers.
// Delete only when stronger recovery coverage subsumes early complete and partial limit refusals.
#[test]
fn impossible_closure_prefixes_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (offset, width, limit, maximum) in [
        (88_usize, 8_usize, Limit::Nodes, 1_048_576_u64),
        (96, 2, Limit::Depth, 8),
        (100, 8, Limit::EncodedBytes, 16_777_216),
        (108, 8, Limit::PhysicalBytes, 1_073_741_824),
    ] {
        for available in 1..width {
            let mut bytes = vec![0; width];
            // A leading 0xff forces every completion above each specified ceiling.
            *bytes.first_mut().ok_or("empty limit")? = 255;
            let minimum = if width == 2 {
                0xff00
            } else {
                0xff00_0000_0000_0000
            };
            let expected = RetentionRootDecodeError::ClosureLimitPrefixAboveMaximum {
                limit,
                minimum,
                maximum,
            };
            require_refusal(
                offset,
                bytes.get(..available).ok_or("missing prefix")?,
                &expected,
            )?;
        }
        for observed in [0, maximum.checked_add(1).ok_or("boundary overflow")?] {
            let raw = observed.to_be_bytes();
            let start = raw.len().checked_sub(width).ok_or("invalid width")?;
            let source = if observed == 0 {
                LimitError::Zero { limit }
            } else {
                LimitError::AboveMaximum {
                    limit,
                    maximum,
                    observed,
                }
            };
            require_refusal(
                offset,
                raw.get(start..).ok_or("missing field")?,
                &RetentionRootDecodeError::ClosureLimit { source },
            )?;
        }
    }
    Ok(())
}

fn require_refusal(
    offset: usize,
    field: &[u8],
    expected: &RetentionRootDecodeError,
) -> Result<(), Box<dyn Error>> {
    let end = offset.checked_add(field.len()).ok_or("offset overflow")?;
    let root = fixture(ROOT_HEX)?;
    let mut partial = root.get(..end).ok_or("missing fixture bytes")?.to_vec();
    partial
        .get_mut(offset..end)
        .ok_or("missing field")?
        .copy_from_slice(field);
    let (sandbox, mut authority) = open_authority(&format!("closure-prefix-{offset}-{end}"))?;
    fs::write(sandbox.path().join("retention/root.next"), partial)?;
    let before = retention_witness(sandbox.path())?;

    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
        }) => {
            assert_eq!(
                stage,
                RetentionFixedStage::Root,
                "identify the invalid closure stage"
            );
            require_cause(source.as_ref(), expected);
        }
        result => {
            return Err(format!("closure prefix {offset}..{end} must refuse: {result:?}").into());
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "closure prefix {offset}..{end} must preserve retained evidence"
    );
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), expected: &RetentionRootDecodeError) {
    let exact = match (source.downcast_ref::<RetentionRootDecodeError>(), expected) {
        (
            Some(RetentionRootDecodeError::ClosureLimitPrefixAboveMaximum {
                limit,
                minimum,
                maximum,
            }),
            RetentionRootDecodeError::ClosureLimitPrefixAboveMaximum {
                limit: expected_limit,
                minimum: expected_minimum,
                maximum: expected_maximum,
            },
        ) => limit == expected_limit && minimum == expected_minimum && maximum == expected_maximum,
        (
            Some(RetentionRootDecodeError::ClosureLimit { source }),
            RetentionRootDecodeError::ClosureLimit { source: expected },
        ) => source == expected,
        _ => false,
    };
    assert!(
        exact,
        "report exact closure-prefix cause: expected {expected:?}, observed {source:?}"
    );
}
