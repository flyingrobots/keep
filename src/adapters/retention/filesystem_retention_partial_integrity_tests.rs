//! This module owns preservation of interrupted records with provably corrupt integrity fields.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionRootDecodeError,
};
use std::{error::Error, fs};

// Size: medium. Oracle: canonical conformance digest/checksum bytes bind the unchanged preimage.
// Delete only when stronger filesystem recovery laws subsume both record trailers at every prefix.
#[test]
fn corrupt_partial_record_trailers_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (stage, corpus) in [(Stage::Root, ROOT_HEX), (Stage::Manifest, MANIFEST_HEX)] {
        let bytes = fixture(corpus)?;
        let digest_start = bytes.len().checked_sub(64).ok_or("missing trailer")?;
        for offset in digest_start..bytes.len().checked_sub(1).ok_or("empty record")? {
            let end = offset.checked_add(1).ok_or("offset overflow")?;
            let mut partial = bytes.get(..end).ok_or("missing prefix")?.to_vec();
            let expected = *partial.get(offset).ok_or("missing trailer byte")?;
            let observed = expected ^ 1;
            *partial.get_mut(offset).ok_or("missing mutation byte")? = observed;
            require_preservation(
                stage,
                &partial,
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

// Size: medium. Oracle: the body-set digest is already determined when the complete body arrives.
// Delete only when stronger public recovery coverage subsumes both body-set digest refusals.
#[test]
fn corrupt_complete_body_digests_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (stage, corpus, offset) in [
        (Stage::Root, ROOT_HEX, 148_usize),
        (Stage::Manifest, MANIFEST_HEX, 80),
    ] {
        let bytes = fixture(corpus)?;
        let body_end = bytes.len().checked_sub(64).ok_or("missing trailer")?;
        let end = offset.checked_add(32).ok_or("digest offset overflow")?;
        let expected = <[u8; 32]>::try_from(bytes.get(offset..end).ok_or("missing digest")?)?;
        let mut observed = expected;
        *observed.first_mut().ok_or("empty digest")? ^= 1;
        let mut partial = bytes.get(..body_end).ok_or("missing body")?.to_vec();
        partial
            .get_mut(offset..end)
            .ok_or("missing header digest")?
            .copy_from_slice(&observed);
        require_preservation(stage, &partial, &Fault::Set { expected, observed })?;
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum Fault {
    Byte {
        offset: usize,
        expected: u8,
        observed: u8,
    },
    Set {
        expected: [u8; 32],
        observed: [u8; 32],
    },
}

fn require_preservation(
    stage: Stage,
    partial: &[u8],
    expected: &Fault,
) -> Result<(), Box<dyn Error>> {
    let (name, phases) = match stage {
        Stage::Root => ("root.next", 1),
        _ => ("manifest.next", 7),
    };
    let (sandbox, mut authority) =
        open_authority(&format!("partial-integrity-{name}-{}", partial.len()))?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, phases)?;
    fs::write(sandbox.path().join("retention").join(name), partial)?;
    let before = retention_witness(sandbox.path())?;
    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source:
                RetentionRecoveryRefusal::StageCorrupt {
                    stage: actual,
                    source,
                },
        }) => {
            assert_eq!(actual, stage, "identify the corrupt integrity stage");
            require_cause(source.as_ref(), stage, expected);
        }
        result => {
            return Err(format!(
                "{name} integrity prefix {} must refuse: {result:?}",
                partial.len()
            )
            .into());
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "corrupt integrity bytes must preserve retained evidence"
    );
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), stage: Stage, expected: &Fault) {
    let actual = match stage {
        Stage::Root => match source.downcast_ref::<RetentionRootDecodeError>() {
            Some(RetentionRootDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }) => Some(Fault::Byte {
                offset: *offset,
                expected: *expected,
                observed: *observed,
            }),
            Some(RetentionRootDecodeError::AnchorSetDigestMismatch { expected, observed }) => {
                Some(Fault::Set {
                    expected: *expected,
                    observed: *observed,
                })
            }
            _ => None,
        },
        _ => match source.downcast_ref::<RetentionManifestDecodeError>() {
            Some(RetentionManifestDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }) => Some(Fault::Byte {
                offset: *offset,
                expected: *expected,
                observed: *observed,
            }),
            Some(RetentionManifestDecodeError::EntrySetDigestMismatch { expected, observed }) => {
                Some(Fault::Set {
                    expected: *expected,
                    observed: *observed,
                })
            }
            _ => None,
        },
    };
    assert_eq!(
        actual.as_ref(),
        Some(expected),
        "report exact available integrity contradiction: {source:?}"
    );
}
