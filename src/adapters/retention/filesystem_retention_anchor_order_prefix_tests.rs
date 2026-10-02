//! This module owns runtime ordering laws for incomplete root anchors.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError, RetentionStageAssessment, assess_root_stage,
};
use std::{error::Error, fs};

struct Case {
    name: &'static str,
    prior: Vec<u8>,
    next: Vec<u8>,
    earliest: usize,
}

// Size: medium. Oracle: anchors strictly order blob length/digest, then layout length/digest.
// Delete only when stronger public recovery laws subsume these decisive partial-order boundaries.
#[test]
fn impossible_partial_anchor_order_preserves_evidence() -> Result<(), Box<dyn Error>> {
    for case in cases()? {
        for length in [case.earliest, 118] {
            let bytes = prefix(&case.prior, &case.next, length)?;
            let (sandbox, mut authority) =
                open_authority(&format!("anchor-order-{}-{length}", case.name))?;
            fs::write(sandbox.path().join("retention/root.next"), bytes)?;
            let before = retention_witness(sandbox.path())?;
            match authority.recover() {
                Err(FilesystemRetentionRecoveryError::Plan {
                    source: RetentionRecoveryRefusal::StageCorrupt { stage, source },
                }) => {
                    assert_eq!(
                        stage,
                        RetentionFixedStage::Root,
                        "identify the corrupt anchor-order stage"
                    );
                    assert!(
                        matches!(
                            source.downcast_ref::<RetentionRootDecodeError>(),
                            Some(RetentionRootDecodeError::NonCanonicalAnchorOrder { index: 1 })
                        ),
                        "report the exact impossible anchor index: {source:?}"
                    );
                }
                result => {
                    return Err(format!(
                        "{} prefix {length} must refuse impossible order: {result:?}",
                        case.name
                    )
                    .into());
                }
            }
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "impossible anchor order must preserve retained evidence"
            );
        }
    }
    Ok(())
}

// Size: small. Oracle: every prefix of a strictly greater complete anchor has that completion.
// Delete only when stronger public assessment laws subsume all lexicographic levels and lengths.
#[test]
fn ordered_anchors_admit_every_partial_length() -> Result<(), Box<dyn Error>> {
    for case in cases()?.into_iter().filter(|case| case.name != "exhausted") {
        for length in 1..119 {
            let bytes = prefix(&case.next, &case.prior, length)?;
            let assessment = assess_root_stage(Some(&bytes));
            assert!(
                matches!(assessment, RetentionStageAssessment::Truncated { .. }),
                "{} ordered prefix {length} must admit completion: {assessment:?}",
                case.name
            );
        }
    }
    Ok(())
}

// Size: small. Oracle: once the most significant differing field forces descent, no suffix repairs it.
// Delete only when stronger runtime laws subsume the full decisive-prefix sweep.
#[test]
fn every_decisive_anchor_order_prefix_is_corrupt() -> Result<(), Box<dyn Error>> {
    for case in cases()? {
        for length in case.earliest..119 {
            let bytes = prefix(&case.prior, &case.next, length)?;
            let assessment = assess_root_stage(Some(&bytes));
            assert!(
                matches!(
                    assessment,
                    RetentionStageAssessment::Corrupt(
                        RetentionRootDecodeError::NonCanonicalAnchorOrder { index: 1 }
                    )
                ),
                "{} descending prefix {length} must refuse: {assessment:?}",
                case.name
            );
        }
    }
    Ok(())
}

fn cases() -> Result<Vec<Case>, Box<dyn Error>> {
    let root = fixture(ROOT_HEX)?;
    let anchor = root.get(195..314).ok_or("missing anchor")?;
    let mut cases = Vec::new();
    for (name, start, end, prior_bytes, next_bytes, earliest) in [
        (
            "blob-length",
            19,
            27,
            0x8000_0000_0000_0000_u64.to_be_bytes().to_vec(),
            0x7fff_ffff_ffff_ffff_u64.to_be_bytes().to_vec(),
            20,
        ),
        ("blob-digest", 27, 59, vec![0x80; 32], vec![0x7f; 32], 28),
        (
            "layout-length",
            79,
            87,
            46_137_520_u64.to_be_bytes().to_vec(),
            176_u64.to_be_bytes().to_vec(),
            84,
        ),
        ("layout-digest", 87, 119, vec![0x80; 32], vec![0x7f; 32], 88),
    ] {
        let mut prior = anchor.to_vec();
        let mut next = anchor.to_vec();
        prior
            .get_mut(start..end)
            .ok_or("missing prior field")?
            .copy_from_slice(&prior_bytes);
        next.get_mut(start..end)
            .ok_or("missing next field")?
            .copy_from_slice(&next_bytes);
        cases.push(Case {
            name,
            prior,
            next,
            earliest,
        });
    }
    let mut maximum = anchor.to_vec();
    maximum
        .get_mut(19..59)
        .ok_or("missing blob coordinate")?
        .fill(255);
    maximum
        .get_mut(79..87)
        .ok_or("missing layout length")?
        .copy_from_slice(&46_137_520_u64.to_be_bytes());
    maximum
        .get_mut(87..119)
        .ok_or("missing layout digest")?
        .fill(255);
    cases.push(Case {
        name: "exhausted",
        prior: maximum.clone(),
        next: maximum,
        earliest: 1,
    });
    Ok(cases)
}

fn prefix(prior: &[u8], next: &[u8], length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let root = fixture(ROOT_HEX)?;
    let mut prefix = root.get(..195).ok_or("missing header")?.to_vec();
    prefix
        .get_mut(24..32)
        .ok_or("missing length")?
        .copy_from_slice(&497_u64.to_be_bytes());
    prefix
        .get_mut(44..48)
        .ok_or("missing count")?
        .copy_from_slice(&2_u32.to_be_bytes());
    prefix.extend_from_slice(prior);
    prefix.extend_from_slice(next.get(..length).ok_or("missing anchor prefix")?);
    Ok(prefix)
}
