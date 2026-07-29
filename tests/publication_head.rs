//! Public publication-head framing and checksum laws.

mod support;

use std::error::Error;

use keep::{
    CatalogGenerationError, CatalogLengthError, ChecksummedPublicationHead,
    PublicationHeadDecodeError,
};
use support::{decode_hex, require_error};

const GENERATION_ONE_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const GENERATION_TWO_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const BUNDLE_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-bundle-head.hex");
const GENERATION_ONE_DIGEST_HEX: &str =
    "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320";
const GENERATION_TWO_DIGEST_HEX: &str =
    "ea7d0055fd21f00ed94809ef4e671d72fa2e6a4a5d9ecefb23f3a320a2dad993";
const BUNDLE_DIGEST_HEX: &str = "0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811";
const MAGIC_OFFSET: usize = 0;
const VERSION_OFFSET: usize = 16;
const FLAGS_OFFSET: usize = 18;
const HEAD_LENGTH_OFFSET: usize = 20;
const CHECKSUM_ALGORITHM_OFFSET: usize = 22;
const DIGEST_ALGORITHM_OFFSET: usize = 23;
const GENERATION_OFFSET: usize = 24;
const CATALOG_LENGTH_OFFSET: usize = 32;
const RESERVED_OFFSET: usize = 72;
const CHECKSUM_OFFSET: usize = 96;

#[test]
fn frozen_publication_heads_are_checksum_verified_exactly() -> Result<(), Box<dyn Error>> {
    let generation_one = head_bytes(GENERATION_ONE_HEX)?;
    let first = ChecksummedPublicationHead::decode(&generation_one)?;
    assert_eq!(first.generation().get(), 1);
    assert_eq!(first.catalog_length().get(), 352);
    assert_eq!(
        first.catalog_digest().as_bytes().as_slice(),
        decode_hex(GENERATION_ONE_DIGEST_HEX)?
    );
    assert_eq!(first.encoded(), generation_one);

    let generation_two = head_bytes(GENERATION_TWO_HEX)?;
    let second = ChecksummedPublicationHead::decode(&generation_two)?;
    assert_eq!(second.generation().get(), 2);
    assert_eq!(second.catalog_length().get(), 352);
    assert_eq!(
        second.catalog_digest().as_bytes().as_slice(),
        decode_hex(GENERATION_TWO_DIGEST_HEX)?
    );

    let bundle = head_bytes(BUNDLE_HEX)?;
    let bundle_head = ChecksummedPublicationHead::decode(&bundle)?;
    assert_eq!(bundle_head.generation().get(), 1);
    assert_eq!(bundle_head.catalog_length().get(), 512);
    assert_eq!(
        bundle_head.catalog_digest().as_bytes().as_slice(),
        decode_hex(BUNDLE_DIGEST_HEX)?
    );
    Ok(())
}

#[test]
fn publication_head_refuses_noncanonical_fixed_fields() -> Result<(), Box<dyn Error>> {
    assert_refusal(MAGIC_OFFSET, 0, |error| {
        matches!(error, PublicationHeadDecodeError::InvalidMagic { .. })
    })?;
    assert_refusal(VERSION_OFFSET + 1, 2, |error| {
        error
            == PublicationHeadDecodeError::UnsupportedVersion {
                expected: 1,
                observed: 2,
            }
    })?;
    assert_refusal(FLAGS_OFFSET + 1, 1, |error| {
        error
            == PublicationHeadDecodeError::Flags {
                expected: 0,
                observed: 1,
            }
    })?;
    assert_refusal(HEAD_LENGTH_OFFSET + 1, 127, |error| {
        error
            == PublicationHeadDecodeError::HeadLength {
                expected: 128,
                observed: 127,
            }
    })?;
    assert_refusal(CHECKSUM_ALGORITHM_OFFSET, 2, |error| {
        error
            == PublicationHeadDecodeError::ChecksumAlgorithm {
                expected: 1,
                observed: 2,
            }
    })?;
    assert_refusal(DIGEST_ALGORITHM_OFFSET, 2, |error| {
        error
            == PublicationHeadDecodeError::DigestAlgorithm {
                expected: 1,
                observed: 2,
            }
    })?;
    assert_refusal(RESERVED_OFFSET, 1, |error| {
        matches!(error, PublicationHeadDecodeError::Reserved { .. })
    })?;
    Ok(())
}

#[test]
fn publication_head_refuses_invalid_generation_and_catalog_length() -> Result<(), Box<dyn Error>> {
    assert_refusal(GENERATION_OFFSET + 7, 0, |error| {
        error
            == PublicationHeadDecodeError::Generation {
                source: CatalogGenerationError::Zero,
            }
    })?;
    assert_u64_refusal(CATALOG_LENGTH_OFFSET, 191, |error| {
        error
            == PublicationHeadDecodeError::CatalogLength {
                source: CatalogLengthError::OutOfBounds {
                    minimum: 192,
                    maximum: 167_772_352,
                    observed: 191,
                },
            }
    })?;
    assert_u64_refusal(CATALOG_LENGTH_OFFSET, 193, |error| {
        error
            == PublicationHeadDecodeError::CatalogLength {
                source: CatalogLengthError::NotCongruent { observed: 193 },
            }
    })?;
    Ok(())
}

#[test]
fn publication_head_refuses_wrong_width_and_checksum() -> Result<(), Box<dyn Error>> {
    let mut encoded = head_bytes(GENERATION_ONE_HEX)?;
    let _last = encoded.pop().ok_or("head fixture is empty")?;
    assert_eq!(
        require_error(
            ChecksummedPublicationHead::decode(&encoded),
            "truncated head was admitted"
        )?,
        PublicationHeadDecodeError::WrongLength {
            expected: 128,
            observed: 127,
        }
    );

    let mut corrupt = head_bytes(GENERATION_ONE_HEX)?;
    let checksum_byte = corrupt
        .get_mut(CHECKSUM_OFFSET)
        .ok_or("head fixture lacks its checksum")?;
    *checksum_byte ^= 1;
    assert!(matches!(
        require_error(
            ChecksummedPublicationHead::decode(&corrupt),
            "corrupt head checksum was admitted"
        )?,
        PublicationHeadDecodeError::ChecksumMismatch { .. }
    ));
    Ok(())
}

fn assert_refusal(
    offset: usize,
    value: u8,
    predicate: impl FnOnce(PublicationHeadDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    let mut encoded = head_bytes(GENERATION_ONE_HEX)?;
    let field = encoded
        .get_mut(offset)
        .ok_or("head fixture lacks the mutation offset")?;
    *field = value;
    let error = require_error(
        ChecksummedPublicationHead::decode(&encoded),
        "mutated head was admitted",
    )?;
    assert!(predicate(error), "unexpected refusal: {error:?}");
    Ok(())
}

fn assert_u64_refusal(
    offset: usize,
    value: u64,
    predicate: impl FnOnce(PublicationHeadDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    let mut encoded = head_bytes(GENERATION_ONE_HEX)?;
    let field = encoded
        .get_mut(offset..offset.checked_add(8).ok_or("test offset overflow")?)
        .ok_or("head fixture lacks the u64 mutation field")?;
    field.copy_from_slice(&value.to_be_bytes());
    let error = require_error(
        ChecksummedPublicationHead::decode(&encoded),
        "mutated head was admitted",
    )?;
    assert!(predicate(error), "unexpected refusal: {error:?}");
    Ok(())
}

fn head_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("head fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
