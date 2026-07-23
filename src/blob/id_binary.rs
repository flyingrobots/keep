//! Canonical binary `BlobId` codec.

use super::id::BlobId;
use super::id_binary_error::BlobIdBinaryParseError;
use super::length::BlobLength;

const BINARY_MAGIC: [u8; 16] = *b"KEEP:BLOB:ID\0\0\0\0";
const IDENTITY_VERSION: u16 = 1;
const HASH_ALGORITHM: u8 = 1;
const BINARY_BYTES: usize = 59;

impl BlobId {
    /// Number of bytes in the canonical version-1 binary representation.
    pub const BINARY_LENGTH: usize = BINARY_BYTES;

    /// Encodes this identity into its fixed canonical binary representation.
    ///
    /// The operation is allocation-free.
    #[must_use]
    pub const fn encode_binary(self) -> [u8; BINARY_BYTES] {
        let mut encoded = [0_u8; BINARY_BYTES];
        let (magic_slot, rest) = encoded.split_at_mut(BINARY_MAGIC.len());
        magic_slot.copy_from_slice(&BINARY_MAGIC);
        let (version_slot, rest) = rest.split_at_mut(2);
        version_slot.copy_from_slice(&IDENTITY_VERSION.to_be_bytes());
        let (algorithm_slot, rest) = rest.split_at_mut(1);
        algorithm_slot.copy_from_slice(&[HASH_ALGORITHM]);
        let (length_slot, digest_slot) = rest.split_at_mut(8);
        length_slot.copy_from_slice(&self.logical_length().get().to_be_bytes());
        digest_slot.copy_from_slice(self.digest());
        encoded
    }

    /// Parses the exact canonical binary representation.
    ///
    /// Parsing validates framing and support. It does not verify possession of
    /// the bytes named by the identity.
    ///
    /// # Errors
    ///
    /// Returns [`BlobIdBinaryParseError`] for truncation, trailing data, wrong
    /// magic, or unsupported identity rules.
    pub fn parse_binary(encoded: &[u8]) -> Result<Self, BlobIdBinaryParseError> {
        validate_binary_length(encoded.len())?;
        let Some((magic, remainder)) = encoded.split_first_chunk::<16>() else {
            return Err(truncated(encoded.len()));
        };
        if magic != &BINARY_MAGIC {
            return Err(BlobIdBinaryParseError::InvalidMagic { observed: *magic });
        }
        let Some((version_bytes, remainder)) = remainder.split_first_chunk::<2>() else {
            return Err(truncated(encoded.len()));
        };
        let observed_version = u16::from_be_bytes(*version_bytes);
        if observed_version != IDENTITY_VERSION {
            return Err(BlobIdBinaryParseError::UnsupportedVersion {
                expected: IDENTITY_VERSION,
                observed: observed_version,
            });
        }
        let Some((algorithm_bytes, remainder)) = remainder.split_first_chunk::<1>() else {
            return Err(truncated(encoded.len()));
        };
        let [observed_algorithm] = *algorithm_bytes;
        if observed_algorithm != HASH_ALGORITHM {
            return Err(BlobIdBinaryParseError::UnsupportedAlgorithm {
                expected: HASH_ALGORITHM,
                observed: observed_algorithm,
            });
        }
        let Some((length_bytes, remainder)) = remainder.split_first_chunk::<8>() else {
            return Err(truncated(encoded.len()));
        };
        let Some((digest, trailing)) = remainder.split_first_chunk::<32>() else {
            return Err(truncated(encoded.len()));
        };
        if !trailing.is_empty() {
            return Err(BlobIdBinaryParseError::TrailingData {
                expected: BINARY_BYTES,
                observed: encoded.len(),
            });
        }
        let logical_length = BlobLength::new(u64::from_be_bytes(*length_bytes));
        Ok(Self::from_validated_parts(logical_length, *digest))
    }
}

fn validate_binary_length(observed: usize) -> Result<(), BlobIdBinaryParseError> {
    match observed.cmp(&BINARY_BYTES) {
        std::cmp::Ordering::Less => Err(truncated(observed)),
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => Err(BlobIdBinaryParseError::TrailingData {
            expected: BINARY_BYTES,
            observed,
        }),
    }
}

const fn truncated(observed: usize) -> BlobIdBinaryParseError {
    BlobIdBinaryParseError::Truncated {
        expected: BINARY_BYTES,
        observed,
    }
}
