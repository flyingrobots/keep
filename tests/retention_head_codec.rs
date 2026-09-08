//! Public semantic and canonical-codec laws for the retention head.

mod support;

use std::io;

use keep::{
    CanonicalRetentionHead, ChecksummedRetentionHead, LivenessGeneration, RetentionHead,
    RetentionHeadDecodeError, RetentionHeadError, RetentionManifestLength,
    RetentionManifestLengthError,
};

const ONE_ROOT_MANIFEST: &str =
    include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
const ONE_ROOT_HEAD: &str = include_str!("../conformance/segment-store/v2/one-root-head.hex");
const CHECKSUM_OFFSET: usize = 112;

#[test]
fn one_root_head_has_one_semantic_and_canonical_representation()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest_bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    let manifest = keep::AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let manifest_length = RetentionManifestLength::new(u64::try_from(manifest_bytes.len())?)?;
    let head = RetentionHead::new(
        manifest.manifest().generation(),
        manifest_length,
        manifest.digest(),
        manifest.manifest().predecessor(),
    )?;

    let canonical = CanonicalRetentionHead::from_head(&head);
    let head_bytes = fixture_bytes(ONE_ROOT_HEAD)?;
    assert_eq!(canonical.encoded(), head_bytes.as_slice());

    let checksummed = ChecksummedRetentionHead::decode(&head_bytes)?;
    assert_eq!(checksummed.encoded(), head_bytes);
    assert_eq!(checksummed.head(), &head);
    Ok(())
}

#[test]
fn manifest_length_and_head_history_are_admitted_exactly() -> Result<(), Box<dyn std::error::Error>>
{
    assert_eq!(RetentionManifestLength::new(224)?.get(), 224);
    assert_eq!(RetentionManifestLength::new(295_136)?.get(), 295_136);
    assert!(matches!(
        RetentionManifestLength::new(223),
        Err(RetentionManifestLengthError::OutOfBounds { .. })
    ));
    assert!(matches!(
        RetentionManifestLength::new(225),
        Err(RetentionManifestLengthError::NotCongruent { .. })
    ));

    let head_bytes = fixture_bytes(ONE_ROOT_HEAD)?;
    let head = ChecksummedRetentionHead::decode(&head_bytes)?;
    assert!(matches!(
        RetentionHead::new(
            LivenessGeneration::new(1)?,
            head.head().manifest_length(),
            head.head().manifest_digest(),
            Some(head.head().manifest_digest()),
        ),
        Err(RetentionHeadError::InitialGenerationHasPredecessor { .. })
    ));
    assert!(matches!(
        RetentionHead::new(
            LivenessGeneration::new(2)?,
            head.head().manifest_length(),
            head.head().manifest_digest(),
            None,
        ),
        Err(RetentionHeadError::MissingPredecessor { .. })
    ));
    Ok(())
}

#[test]
fn head_framing_and_integrity_have_exact_first_refusals() -> Result<(), Box<dyn std::error::Error>>
{
    let bytes = fixture_bytes(ONE_ROOT_HEAD)?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        ChecksummedRetentionHead::decode(&truncated),
        Err(RetentionHeadDecodeError::WrongLength {
            expected: 144,
            observed: 143,
        })
    ));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        ChecksummedRetentionHead::decode(&trailing),
        Err(RetentionHeadDecodeError::WrongLength {
            expected: 144,
            observed: 145,
        })
    ));

    let mut wrong_magic = bytes.clone();
    let first = wrong_magic
        .first_mut()
        .ok_or_else(|| io::Error::other("frozen retention head is empty"))?;
    *first ^= 1;
    assert!(matches!(
        ChecksummedRetentionHead::decode(&wrong_magic),
        Err(RetentionHeadDecodeError::InvalidMagic { .. })
    ));

    let mut checksum_corruption = bytes;
    let last = checksum_corruption
        .last_mut()
        .ok_or_else(|| io::Error::other("frozen retention head is empty"))?;
    *last ^= 1;
    assert!(matches!(
        ChecksummedRetentionHead::decode(&checksum_corruption),
        Err(RetentionHeadDecodeError::ChecksumMismatch { .. })
    ));
    Ok(())
}

#[test]
fn complete_integrity_precedes_head_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes(ONE_ROOT_HEAD)?;
    bytes
        .get_mut(24..32)
        .ok_or_else(|| io::Error::other("frozen retention head lacks generation bytes"))?
        .fill(0);
    assert!(matches!(
        ChecksummedRetentionHead::decode(&bytes),
        Err(RetentionHeadDecodeError::ChecksumMismatch { .. })
    ));

    refresh_checksum(&mut bytes)?;
    assert!(matches!(
        ChecksummedRetentionHead::decode(&bytes),
        Err(RetentionHeadDecodeError::LivenessGeneration { .. })
    ));

    let mut noncanonical_length = fixture_bytes(ONE_ROOT_HEAD)?;
    noncanonical_length
        .get_mut(32..40)
        .ok_or_else(|| io::Error::other("frozen retention head lacks manifest length bytes"))?
        .copy_from_slice(&225_u64.to_be_bytes());
    refresh_checksum(&mut noncanonical_length)?;
    assert!(matches!(
        ChecksummedRetentionHead::decode(&noncanonical_length),
        Err(RetentionHeadDecodeError::ManifestLength { .. })
    ));

    let mut missing_predecessor = fixture_bytes(ONE_ROOT_HEAD)?;
    missing_predecessor
        .get_mut(24..32)
        .ok_or_else(|| io::Error::other("frozen retention head lacks generation bytes"))?
        .copy_from_slice(&2_u64.to_be_bytes());
    refresh_checksum(&mut missing_predecessor)?;
    assert!(matches!(
        ChecksummedRetentionHead::decode(&missing_predecessor),
        Err(RetentionHeadDecodeError::Semantic {
            source: RetentionHeadError::MissingPredecessor { .. },
        })
    ));
    Ok(())
}

fn fixture_bytes(fixture: &str) -> Result<Vec<u8>, io::Error> {
    let encoded = fixture
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("retention fixture lacks final newline"))?;
    support::decode_hex(encoded)
}

fn refresh_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let (preimage, trailer) = bytes
        .split_at_mut_checked(CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("retention head lacks its checksum"))?;
    let checksum = trailer
        .get_mut(..blake3::OUT_LEN)
        .ok_or_else(|| io::Error::other("retention head checksum is truncated"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.retention-head-checksum/v2\0");
    hasher.update(preimage);
    checksum.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}
