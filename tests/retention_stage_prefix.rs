//! Recovery assessment obeys the version-two retention fixed-field grammar.
//! Size: small; oracle: conformance records and the retention format grammar.
//! Delete only when a stronger public-contract test subsumes these prefix laws.

mod support;

use std::error::Error;

use keep::{RetentionStageAssessment, assess_head_stage, assess_manifest_stage, assess_root_stage};

const ROOT: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const MANIFEST: &str = include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
const HEAD: &str = include_str!("../conformance/segment-store/v2/one-root-head.hex");

#[test]
fn canonical_root_prefixes_remain_recoverable_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(ROOT.trim_end())?;
    for end in 0..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing root prefix")?;
        let assessment = assess_root_stage(Some(prefix));
        assert!(
            matches!(assessment, RetentionStageAssessment::Truncated { .. }),
            "root prefix {end} must be truncated, observed {assessment:?}"
        );
    }
    Ok(())
}

#[test]
fn canonical_manifest_prefixes_remain_recoverable_interrupted_writes() -> Result<(), Box<dyn Error>>
{
    let bytes = support::decode_hex(MANIFEST.trim_end())?;
    for end in 0..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing manifest prefix")?;
        let assessment = assess_manifest_stage(Some(prefix));
        assert!(
            matches!(assessment, RetentionStageAssessment::Truncated { .. }),
            "manifest prefix {end} must be truncated, observed {assessment:?}"
        );
    }
    Ok(())
}

#[test]
fn canonical_head_prefixes_remain_recoverable_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(HEAD.trim_end())?;
    for end in 0..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing head prefix")?;
        let assessment = assess_head_stage(Some(prefix));
        assert!(
            matches!(assessment, RetentionStageAssessment::Truncated { .. }),
            "head prefix {end} must be truncated, observed {assessment:?}"
        );
    }
    Ok(())
}

#[test]
fn contradictory_root_fixed_bytes_are_corrupt_at_every_interrupted_length()
-> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(ROOT.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(42..44).chain(98..100).chain(180..192) {
        let mut altered = bytes.clone();
        *altered.get_mut(offset).ok_or("missing root fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing root prefix")?;
            let assessment = assess_root_stage(Some(prefix));
            assert!(
                matches!(assessment, RetentionStageAssessment::Corrupt(_)),
                "root byte {offset}, prefix {end} must be corrupt, observed {assessment:?}"
            );
            witnesses += 1;
        }
    }
    assert!(
        witnesses > 0,
        "root corruption law must examine actual prefixes"
    );
    Ok(())
}

#[test]
fn contradictory_manifest_fixed_bytes_are_corrupt_at_every_interrupted_length()
-> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(MANIFEST.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(40..44).chain(112..160) {
        let mut altered = bytes.clone();
        *altered
            .get_mut(offset)
            .ok_or("missing manifest fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing manifest prefix")?;
            let assessment = assess_manifest_stage(Some(prefix));
            assert!(
                matches!(assessment, RetentionStageAssessment::Corrupt(_)),
                "manifest byte {offset}, prefix {end} must be corrupt, observed {assessment:?}"
            );
            witnesses += 1;
        }
    }
    assert!(
        witnesses > 0,
        "manifest corruption law must examine actual prefixes"
    );
    Ok(())
}

#[test]
fn contradictory_head_fixed_bytes_are_corrupt_at_every_interrupted_length()
-> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(HEAD.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(104..112) {
        let mut altered = bytes.clone();
        *altered.get_mut(offset).ok_or("missing head fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing head prefix")?;
            let assessment = assess_head_stage(Some(prefix));
            assert!(
                matches!(assessment, RetentionStageAssessment::Corrupt(_)),
                "head byte {offset}, prefix {end} must be corrupt, observed {assessment:?}"
            );
            witnesses += 1;
        }
    }
    assert!(
        witnesses > 0,
        "head corruption law must examine actual prefixes"
    );
    Ok(())
}

#[test]
fn short_root_refusal_reports_only_observed_bytes() {
    let assessment = assess_root_stage(Some(b"X"));
    assert!(
        matches!(
            assessment,
            RetentionStageAssessment::Corrupt(keep::RetentionRootDecodeError::PrefixByteMismatch {
                offset: 0,
                expected: b'K',
                observed: b'X',
            })
        ),
        "one-byte root refusal must report offset 0, K and X; observed {assessment:?}"
    );
}

#[test]
fn short_manifest_refusal_reports_only_observed_bytes() {
    let assessment = assess_manifest_stage(Some(b"X"));
    assert!(
        matches!(
            assessment,
            RetentionStageAssessment::Corrupt(
                keep::RetentionManifestDecodeError::PrefixByteMismatch {
                    offset: 0,
                    expected: b'K',
                    observed: b'X',
                }
            )
        ),
        "one-byte manifest refusal must report offset 0, K and X; observed {assessment:?}"
    );
}

#[test]
fn short_head_refusal_reports_only_observed_bytes() {
    let assessment = assess_head_stage(Some(b"X"));
    assert!(
        matches!(
            assessment,
            RetentionStageAssessment::Corrupt(keep::RetentionHeadDecodeError::PrefixByteMismatch {
                offset: 0,
                expected: b'K',
                observed: b'X',
            })
        ),
        "one-byte head refusal must report offset 0, K and X; observed {assessment:?}"
    );
}

#[test]
fn admitted_closure_values_remain_possible_at_every_prefix() -> Result<(), Box<dyn Error>> {
    let fixture = support::decode_hex(ROOT.trim_end())?;
    for (offset, width, maximum) in [
        (88_usize, 8_usize, 1_048_576_u64),
        (96, 2, 8),
        (100, 8, 16_777_216),
        (108, 8, 1_073_741_824),
    ] {
        // Deterministically cover every admitted high-bit boundary, plus the ceiling.
        for value in (0..64)
            .filter_map(|bit| 1_u64.checked_shl(bit))
            .filter(|value| *value <= maximum)
            .chain([maximum])
        {
            let raw = value.to_be_bytes();
            let start = raw.len().checked_sub(width).ok_or("field width overflow")?;
            let field = raw.get(start..).ok_or("missing encoded field")?;
            for length in 1..=width {
                let end = offset.checked_add(length).ok_or("prefix overflow")?;
                let mut prefix = fixture.get(..end).ok_or("missing fixture")?.to_vec();
                prefix
                    .get_mut(offset..end)
                    .ok_or("missing target")?
                    .copy_from_slice(field.get(..length).ok_or("missing source")?);
                let assessment = assess_root_stage(Some(&prefix));
                assert!(
                    matches!(assessment, RetentionStageAssessment::Truncated { .. }),
                    "admitted closure value {value} at {offset}, prefix length {length}, must admit completion: {assessment:?}"
                );
            }
        }
    }
    Ok(())
}
