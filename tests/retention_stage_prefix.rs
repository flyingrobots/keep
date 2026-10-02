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
        assert!(matches!(assess_root_stage(Some(prefix)), RetentionStageAssessment::Truncated { .. }), "root prefix {end}");
    }
    Ok(())
}

#[test]
fn canonical_manifest_prefixes_remain_recoverable_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(MANIFEST.trim_end())?;
    for end in 0..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing manifest prefix")?;
        assert!(matches!(assess_manifest_stage(Some(prefix)), RetentionStageAssessment::Truncated { .. }), "manifest prefix {end}");
    }
    Ok(())
}

#[test]
fn canonical_head_prefixes_remain_recoverable_interrupted_writes() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(HEAD.trim_end())?;
    for end in 0..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing head prefix")?;
        assert!(matches!(assess_head_stage(Some(prefix)), RetentionStageAssessment::Truncated { .. }), "head prefix {end}");
    }
    Ok(())
}

#[test]
fn contradictory_root_fixed_bytes_are_corrupt_at_every_interrupted_length() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(ROOT.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(42..44).chain(98..100).chain(180..192) {
        let mut altered = bytes.clone();
        *altered.get_mut(offset).ok_or("missing root fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing root prefix")?;
            assert!(matches!(assess_root_stage(Some(prefix)), RetentionStageAssessment::Corrupt(_)), "root byte {offset}, prefix {end}");
            witnesses += 1;
        }
    }
    assert!(witnesses > 0, "root corruption law must examine actual prefixes");
    Ok(())
}

#[test]
fn contradictory_manifest_fixed_bytes_are_corrupt_at_every_interrupted_length() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(MANIFEST.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(40..44).chain(112..160) {
        let mut altered = bytes.clone();
        *altered.get_mut(offset).ok_or("missing manifest fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing manifest prefix")?;
            assert!(matches!(assess_manifest_stage(Some(prefix)), RetentionStageAssessment::Corrupt(_)), "manifest byte {offset}, prefix {end}");
            witnesses += 1;
        }
    }
    assert!(witnesses > 0, "manifest corruption law must examine actual prefixes");
    Ok(())
}

#[test]
fn contradictory_head_fixed_bytes_are_corrupt_at_every_interrupted_length() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(HEAD.trim_end())?;
    let mut witnesses = 0;
    for offset in (0..24).chain(104..112) {
        let mut altered = bytes.clone();
        *altered.get_mut(offset).ok_or("missing head fixed byte")? ^= 1;
        for end in offset.checked_add(1).ok_or("prefix overflow")?..bytes.len() {
            let prefix = altered.get(..end).ok_or("missing head prefix")?;
            assert!(matches!(assess_head_stage(Some(prefix)), RetentionStageAssessment::Corrupt(_)), "head byte {offset}, prefix {end}");
            witnesses += 1;
        }
    }
    assert!(witnesses > 0, "head corruption law must examine actual prefixes");
    Ok(())
}
