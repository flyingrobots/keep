//! Canonical version-2 store-format marker laws.

mod support;

use std::io;

use keep::{
    AdmittedStoreFormatMarker, CanonicalStoreFormatMarker, StoreFormatDefinitionDigest,
    StoreFormatMarkerDecodeError,
};

const FORMAT_MARKER: &str = include_str!("../conformance/segment-store/v2/format-marker.hex");
const MARKER_DIGEST: [u8; 32] = [
    0x4b, 0x06, 0x3c, 0x32, 0x90, 0x85, 0xab, 0xde, 0xbe, 0x86, 0xb2, 0x56, 0xd5, 0x31, 0xb1, 0x12,
    0xc7, 0xea, 0x33, 0xcb, 0x2f, 0x54, 0x5c, 0xaa, 0x40, 0xa7, 0xa8, 0x69, 0xff, 0x33, 0x37, 0xce,
];

#[test]
fn marker_reproduces_the_frozen_version_two_record() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let admitted = AdmittedStoreFormatMarker::decode(&bytes)?;
    let canonical = CanonicalStoreFormatMarker::version_two();

    assert_eq!(admitted.encoded(), bytes);
    assert_eq!(
        admitted.definition_digest(),
        StoreFormatDefinitionDigest::VERSION_TWO
    );
    assert_eq!(admitted.digest().as_bytes(), &MARKER_DIGEST);
    assert_eq!(canonical.encoded(), bytes);
    assert_eq!(canonical.digest(), admitted.digest());
    Ok(())
}

#[test]
fn marker_framing_has_exact_first_refusals() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedStoreFormatMarker::decode(&truncated),
        Err(StoreFormatMarkerDecodeError::WrongLength {
            expected: 96,
            observed: 95,
        })
    ));

    assert_fixed_refusal(
        0,
        StoreFormatMarkerDecodeError::InvalidMagic {
            observed: mutated_array::<16>(&bytes, 0, 0)?,
        },
    )?;
    assert_fixed_refusal(
        17,
        StoreFormatMarkerDecodeError::UnsupportedVersion {
            expected: 2,
            observed: 3,
        },
    )?;
    assert_fixed_refusal(
        19,
        StoreFormatMarkerDecodeError::InvalidRecordLength {
            expected: 96,
            observed: 97,
        },
    )?;
    assert_fixed_refusal(
        23,
        StoreFormatMarkerDecodeError::UnsupportedFlags { observed: 1 },
    )?;
    assert_fixed_refusal(
        63,
        StoreFormatMarkerDecodeError::NonZeroReserved { observed: 1 },
    )?;
    Ok(())
}

#[test]
fn checksum_precedes_registered_marker_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let mut definition = fixture_bytes()?;
    flip_byte(&mut definition, 24)?;
    assert!(matches!(
        AdmittedStoreFormatMarker::decode(&definition),
        Err(StoreFormatMarkerDecodeError::ChecksumMismatch { .. })
    ));
    refresh_checksum(&mut definition)?;
    assert!(matches!(
        AdmittedStoreFormatMarker::decode(&definition),
        Err(StoreFormatMarkerDecodeError::DefinitionDigestMismatch { .. })
    ));

    let mut namespace_limit = fixture_bytes()?;
    namespace_limit
        .get_mut(56..60)
        .ok_or_else(|| io::Error::other("marker lacks namespace limit"))?
        .copy_from_slice(&4_095_u32.to_be_bytes());
    refresh_checksum(&mut namespace_limit)?;
    assert_eq!(
        AdmittedStoreFormatMarker::decode(&namespace_limit),
        Err(StoreFormatMarkerDecodeError::InvalidMaximumNamespaceCount {
            expected: 4_096,
            observed: 4_095,
        })
    );

    let mut checksum = fixture_bytes()?;
    flip_byte(&mut checksum, 95)?;
    assert!(matches!(
        AdmittedStoreFormatMarker::decode(&checksum),
        Err(StoreFormatMarkerDecodeError::ChecksumMismatch { .. })
    ));
    Ok(())
}

fn assert_fixed_refusal(
    offset: usize,
    expected: StoreFormatMarkerDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    flip_byte(&mut bytes, offset)?;
    assert_eq!(AdmittedStoreFormatMarker::decode(&bytes), Err(expected));
    Ok(())
}

fn mutated_array<const WIDTH: usize>(
    bytes: &[u8],
    offset: usize,
    relative: usize,
) -> Result<[u8; WIDTH], io::Error> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or_else(|| io::Error::other("marker field offset overflow"))?;
    let mut observed = <[u8; WIDTH]>::try_from(
        bytes
            .get(offset..end)
            .ok_or_else(|| io::Error::other("marker lacks fixed field"))?,
    )
    .map_err(|_| io::Error::other("marker field width mismatch"))?;
    let byte = observed
        .get_mut(relative)
        .ok_or_else(|| io::Error::other("marker mutation offset is out of bounds"))?;
    *byte ^= 1;
    Ok(observed)
}

fn flip_byte(bytes: &mut [u8], offset: usize) -> Result<(), io::Error> {
    let byte = bytes
        .get_mut(offset)
        .ok_or_else(|| io::Error::other("marker mutation offset is out of bounds"))?;
    *byte ^= 1;
    Ok(())
}

fn refresh_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let (preimage, checksum) = bytes
        .split_at_mut_checked(64)
        .ok_or_else(|| io::Error::other("marker lacks checksum boundary"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.segment-store-marker-checksum/v2\0");
    hasher.update(preimage);
    checksum.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}

fn fixture_bytes() -> Result<Vec<u8>, io::Error> {
    support::decode_hex(FORMAT_MARKER.trim_end())
}
