//! This module owns preservation of interrupted roots with invalid complete policies.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError,
};
use crate::{
    RegisteredRetentionProfile, RetentionClosureLimit as Limit,
    RetentionClosureLimitError as LimitError, RetentionProfileAdmissionError as ProfileError,
};
use std::{error::Error, fs};

// Size: medium. Oracle: registered profile coordinates and digest are exact protocol requirements.
// Delete only when stronger public recovery coverage subsumes these malformed profiles.
#[test]
fn invalid_short_root_profiles_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (offset, replacement, expected) in [
        (
            48,
            2_u32.to_be_bytes().to_vec(),
            ProfileError::UnsupportedCoordinate {
                expected_identity: 1,
                expected_version: 1,
                observed_identity: 2,
                observed_version: 1,
            },
        ),
        (
            52,
            2_u32.to_be_bytes().to_vec(),
            ProfileError::UnsupportedCoordinate {
                expected_identity: 1,
                expected_version: 1,
                observed_identity: 1,
                observed_version: 2,
            },
        ),
        (
            56,
            vec![0; 32],
            ProfileError::DefinitionDigestMismatch {
                expected: *RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1.digest(),
                observed: [0; 32],
            },
        ),
    ] {
        require_policy_refusal(
            offset,
            &replacement,
            88,
            &RetentionRootDecodeError::Profile { source: expected },
        )?;
    }
    Ok(())
}

// Size: medium. Oracle: each closure resource is positive and bounded by the v2 format ceiling.
// Delete only when stronger public recovery coverage subsumes all resource violations.
#[test]
fn invalid_short_root_closure_limits_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (offset, width, limit, maximum) in [
        (88, 8, Limit::Nodes, 1_048_576_u64),
        (96, 2, Limit::Depth, 8),
        (100, 8, Limit::EncodedBytes, 16_777_216),
        (108, 8, Limit::PhysicalBytes, 1_073_741_824),
    ] {
        for observed in [0, maximum.checked_add(1).ok_or("test boundary overflow")?] {
            let bytes = observed.to_be_bytes();
            let start = bytes
                .len()
                .checked_sub(width)
                .ok_or("invalid field width")?;
            let replacement = bytes.get(start..).ok_or("missing replacement")?;
            let source = if observed == 0 {
                LimitError::Zero { limit }
            } else {
                LimitError::AboveMaximum {
                    limit,
                    maximum,
                    observed,
                }
            };
            require_policy_refusal(
                offset,
                replacement,
                116,
                &RetentionRootDecodeError::ClosureLimit { source },
            )?;
        }
    }
    Ok(())
}

fn require_policy_refusal(
    offset: usize,
    replacement: &[u8],
    prefix_length: usize,
    expected: &RetentionRootDecodeError,
) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) =
        open_authority(&format!("short-root-policy-{offset}-{prefix_length}"))?;
    let bytes = fixture(ROOT_HEX)?;
    let mut partial = bytes.get(..prefix_length).ok_or("short fixture")?.to_vec();
    let end = offset
        .checked_add(replacement.len())
        .ok_or("test offset overflow")?;
    partial
        .get_mut(offset..end)
        .ok_or("missing policy field")?
        .copy_from_slice(replacement);
    fs::write(sandbox.path().join("retention/root.next"), partial)?;
    let before = retention_witness(sandbox.path())?;

    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
        }) => {
            assert_eq!(
                stage,
                RetentionFixedStage::Root,
                "identify the corrupt policy stage"
            );
            require_policy_cause(source.as_ref(), expected);
        }
        result => {
            return Err(format!(
                "invalid root policy at {offset} must refuse recovery: {result:?}"
            )
            .into());
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "invalid policy must preserve retained evidence"
    );
    Ok(())
}

fn require_policy_cause(source: &(dyn Error + 'static), expected: &RetentionRootDecodeError) {
    let exact = match (source.downcast_ref::<RetentionRootDecodeError>(), expected) {
        (
            Some(RetentionRootDecodeError::Profile { source: actual }),
            RetentionRootDecodeError::Profile { source: expected },
        ) => actual == expected,
        (
            Some(RetentionRootDecodeError::ClosureLimit { source: actual }),
            RetentionRootDecodeError::ClosureLimit { source: expected },
        ) => actual == expected,
        _ => false,
    };
    assert!(
        exact,
        "report exact root policy violation: expected {expected:?}, observed {source:?}"
    );
}
