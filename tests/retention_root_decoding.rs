//! Public decoding and integrity laws for version-2 retention roots.

mod support;

use std::io;

use keep::{AdmittedRetentionRoot, RetentionRootDecodeError};

const ONE_ANCHOR_ROOT: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const ANCHOR_SET_DIGEST_OFFSET: usize = 148;
const ANCHOR_SET_DIGEST_END: usize = 180;
const ANCHOR_BODY_OFFSET: usize = 195;
const ROOT_DIGEST_OFFSET: usize = 314;
const CHECKSUM_OFFSET: usize = 346;

#[test]
fn frozen_root_decodes_to_one_complete_semantic_generation()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let admitted = AdmittedRetentionRoot::decode(&bytes)?;
    assert_eq!(admitted.encoded(), bytes);
    assert_eq!(admitted.root().namespace().as_bytes(), &[0x00, 0x2f, 0xff]);
    assert_eq!(admitted.root().generation().get(), 1);
    assert_eq!(admitted.root().anchor_count(), 1);
    assert_eq!(
        admitted.anchor_set_digest().as_bytes(),
        bytes
            .get(ANCHOR_SET_DIGEST_OFFSET..ANCHOR_SET_DIGEST_END)
            .ok_or_else(|| io::Error::other("frozen root lacks its anchor-set digest"))?
    );
    assert_eq!(
        admitted.digest().as_bytes(),
        bytes.get(314..346).ok_or_else(|| {
            io::Error::other("frozen retention root lacks its embedded digest")
        })?
    );
    Ok(())
}

#[test]
fn root_framing_refuses_truncation_trailing_data_and_magic_substitution()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedRetentionRoot::decode(&truncated),
        Err(RetentionRootDecodeError::Truncated {
            expected: 378,
            observed: 377,
        })
    ));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        AdmittedRetentionRoot::decode(&trailing),
        Err(RetentionRootDecodeError::TrailingData {
            expected: 378,
            observed: 379,
        })
    ));

    let mut wrong_magic = bytes;
    let first = wrong_magic
        .first_mut()
        .ok_or_else(|| io::Error::other("frozen retention root is empty"))?;
    *first ^= 1;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&wrong_magic),
        Err(RetentionRootDecodeError::InvalidMagic { .. })
    ));
    Ok(())
}

#[test]
fn root_checksum_and_digest_have_distinct_integrity_refusals()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let mut checksum_corruption = bytes.clone();
    let last = checksum_corruption
        .last_mut()
        .ok_or_else(|| io::Error::other("frozen retention root is empty"))?;
    *last ^= 1;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&checksum_corruption),
        Err(RetentionRootDecodeError::ChecksumMismatch { .. })
    ));

    let mut digest_corruption = bytes;
    let digest_byte = digest_corruption
        .get_mut(314)
        .ok_or_else(|| io::Error::other("frozen retention root lacks digest bytes"))?;
    *digest_byte ^= 1;
    refresh_checksum(&mut digest_corruption)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&digest_corruption),
        Err(RetentionRootDecodeError::RootDigestMismatch { .. })
    ));
    Ok(())
}

#[test]
fn semantic_fields_are_admitted_only_after_complete_integrity()
-> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    bytes
        .get_mut(32..40)
        .ok_or_else(|| io::Error::other("frozen retention root lacks generation bytes"))?
        .fill(0);
    assert!(matches!(
        AdmittedRetentionRoot::decode(&bytes),
        Err(RetentionRootDecodeError::ChecksumMismatch { .. })
    ));

    refresh_root_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&bytes),
        Err(RetentionRootDecodeError::Generation { .. })
    ));
    Ok(())
}

#[test]
fn anchor_set_integrity_precedes_nested_identity_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    let first_anchor_byte = bytes
        .get_mut(ANCHOR_BODY_OFFSET)
        .ok_or_else(|| io::Error::other("frozen retention root lacks its anchor body"))?;
    *first_anchor_byte ^= 1;
    refresh_root_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&bytes),
        Err(RetentionRootDecodeError::AnchorSetDigestMismatch { .. })
    ));

    refresh_anchor_set_digest(&mut bytes)?;
    refresh_root_digest_and_checksum(&mut bytes)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&bytes),
        Err(RetentionRootDecodeError::BlobId { index: 0, .. })
    ));
    Ok(())
}

fn fixture_bytes() -> Result<Vec<u8>, io::Error> {
    let encoded = ONE_ANCHOR_ROOT
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("retention root fixture lacks final newline"))?;
    support::decode_hex(encoded)
}

fn refresh_anchor_set_digest(bytes: &mut [u8]) -> Result<(), io::Error> {
    let anchors = bytes
        .get(ANCHOR_BODY_OFFSET..ROOT_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("retention root lacks its anchor body"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-anchor-set/v2\0");
    hasher.update(&1_u32.to_be_bytes());
    hasher.update(anchors);
    let digest = *hasher.finalize().as_bytes();
    bytes
        .get_mut(ANCHOR_SET_DIGEST_OFFSET..ANCHOR_SET_DIGEST_OFFSET + 32)
        .ok_or_else(|| io::Error::other("retention root lacks its anchor-set digest"))?
        .copy_from_slice(&digest);
    Ok(())
}

fn refresh_root_digest_and_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let preimage = bytes
        .get(..ROOT_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("retention root lacks its root digest preimage"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-root/v2\0");
    hasher.update(preimage);
    let digest = *hasher.finalize().as_bytes();
    bytes
        .get_mut(ROOT_DIGEST_OFFSET..CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("retention root lacks its root digest"))?
        .copy_from_slice(&digest);
    refresh_checksum(bytes)
}

fn refresh_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("retention root lacks a checksum"))?;
    let (preimage, checksum_slot) = bytes.split_at_mut(checksum_offset);
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-root-checksum/v2\0");
    hasher.update(preimage);
    checksum_slot.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}
