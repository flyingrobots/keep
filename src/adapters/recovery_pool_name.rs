//! This module owns canonical immutable-pool name parsing.

use super::lower_hex::{LowerHexError, decode_digest_32};
use super::{RecoveryEntryName, RecoveryPoolNameError, SegmentDigest};
use crate::{CatalogDigest, CatalogGeneration};

const SEGMENT_NAME_LENGTH: usize = 68;
const CATALOG_NAME_LENGTH: usize = 85;
const GENERATION_LENGTH: usize = 16;

pub(super) fn segment(name: &RecoveryEntryName) -> Result<SegmentDigest, RecoveryPoolNameError> {
    let bytes = name.as_bytes();
    require_length(bytes, SEGMENT_NAME_LENGTH)?;
    let digest = bytes
        .strip_suffix(b".seg")
        .ok_or(RecoveryPoolNameError::WrongSuffix)?;
    decode_digest(digest).map(SegmentDigest::from_validated)
}

pub(super) fn catalog(
    name: &RecoveryEntryName,
) -> Result<(CatalogGeneration, CatalogDigest), RecoveryPoolNameError> {
    let bytes = name.as_bytes();
    require_length(bytes, CATALOG_NAME_LENGTH)?;
    let stem = bytes
        .strip_suffix(b".cat")
        .ok_or(RecoveryPoolNameError::WrongSuffix)?;
    let generation_bytes = stem
        .get(..GENERATION_LENGTH)
        .ok_or(RecoveryPoolNameError::WrongSeparator)?;
    if stem.get(GENERATION_LENGTH) != Some(&b'-') {
        return Err(RecoveryPoolNameError::WrongSeparator);
    }
    let digest_bytes = stem
        .get(GENERATION_LENGTH + 1..)
        .ok_or(RecoveryPoolNameError::WrongSeparator)?;
    let generation = decode_generation(generation_bytes)?;
    let digest = CatalogDigest::from_validated(decode_digest(digest_bytes)?);
    Ok((generation, digest))
}

const fn require_length(bytes: &[u8], expected: usize) -> Result<(), RecoveryPoolNameError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(RecoveryPoolNameError::WrongLength {
            expected,
            observed: bytes.len(),
        })
    }
}

fn decode_generation(bytes: &[u8]) -> Result<CatalogGeneration, RecoveryPoolNameError> {
    if bytes
        .iter()
        .copied()
        .any(|byte| matches!(byte, b'A'..=b'F'))
    {
        return Err(RecoveryPoolNameError::UppercaseGeneration);
    }
    if !bytes.iter().all(u8::is_ascii_hexdigit) {
        return Err(RecoveryPoolNameError::InvalidGenerationAlphabet);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| RecoveryPoolNameError::InvalidGenerationAlphabet)?;
    let value = u64::from_str_radix(text, 16)
        .map_err(|_| RecoveryPoolNameError::InvalidGenerationAlphabet)?;
    CatalogGeneration::new(value).map_err(|_| RecoveryPoolNameError::ZeroGeneration)
}

fn decode_digest(bytes: &[u8]) -> Result<[u8; 32], RecoveryPoolNameError> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| RecoveryPoolNameError::InvalidDigestAlphabet)?;
    decode_digest_32(text).map_err(|source| match source {
        LowerHexError::WrongLength { expected, observed } => {
            RecoveryPoolNameError::DigestLength { expected, observed }
        }
        LowerHexError::Uppercase => RecoveryPoolNameError::UppercaseDigest,
        LowerHexError::InvalidAlphabet => RecoveryPoolNameError::InvalidDigestAlphabet,
    })
}
