//! Identity-specific conformance assertions.

use keep::{BlobHasher, BlobId, BlobIdTextParseError};

use super::harness_failure::HarnessFailure;
use super::scenario_corpus::TextErrorClass;

pub(super) fn hash_partitioned(bytes: &[u8], widths: &[usize]) -> Result<BlobId, HarnessFailure> {
    if widths.is_empty() || widths.contains(&0) {
        return Err(HarnessFailure::corpus(
            "partition plan contains no progress",
        ));
    }
    let mut remaining = bytes;
    let mut hasher = BlobHasher::new();
    for width in widths.iter().cycle() {
        if remaining.is_empty() {
            break;
        }
        let count = remaining.len().min(*width);
        let Some((chunk, rest)) = remaining.split_at_checked(count) else {
            return Err(HarnessFailure::corpus("partition escaped fixture"));
        };
        hasher.update(chunk)?;
        remaining = rest;
    }
    Ok(hasher.finish())
}

pub(super) fn assert_named_bytes(expected: BlobId, bytes: &[u8]) -> Result<(), HarnessFailure> {
    let observed = BlobId::hash_bytes(bytes)?;
    if observed == expected {
        Ok(())
    } else {
        Err(HarnessFailure::NamedBytesMismatch { expected, observed })
    }
}

pub(super) fn generated_bytes(seed: u64, length: usize) -> Vec<u8> {
    let mut state = seed;
    let mut bytes = Vec::with_capacity(length);
    for _ in 0..length {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let [byte, ..] = state.to_be_bytes();
        bytes.push(byte);
    }
    bytes
}

pub(super) const fn text_error_class(error: BlobIdTextParseError) -> TextErrorClass {
    match error {
        BlobIdTextParseError::InputTooLong { .. } => TextErrorClass::InputTooLong,
        BlobIdTextParseError::MissingField => TextErrorClass::MissingField,
        BlobIdTextParseError::TrailingData => TextErrorClass::TrailingData,
        BlobIdTextParseError::InvalidScheme => TextErrorClass::InvalidScheme,
        BlobIdTextParseError::InvalidKind => TextErrorClass::InvalidKind,
        BlobIdTextParseError::MalformedVersion => TextErrorClass::MalformedVersion,
        BlobIdTextParseError::UnsupportedVersion { .. } => TextErrorClass::UnsupportedVersion,
        BlobIdTextParseError::UnsupportedAlgorithm => TextErrorClass::UnsupportedAlgorithm,
        BlobIdTextParseError::NonCanonicalLength => TextErrorClass::NonCanonicalLength,
        BlobIdTextParseError::LengthOverflow => TextErrorClass::LengthOverflow,
        BlobIdTextParseError::InvalidDigestLength { .. } => TextErrorClass::InvalidDigestLength,
        BlobIdTextParseError::NonCanonicalDigestCase => TextErrorClass::NonCanonicalDigestCase,
        BlobIdTextParseError::InvalidDigestAlphabet => TextErrorClass::InvalidDigestAlphabet,
    }
}
