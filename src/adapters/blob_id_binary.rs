//! Canonical binary `BlobId` codec.

use super::blob_id_binary_error::BlobIdBinaryParseError;
use crate::blob::{BlobId, BlobLength};

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
        let [
            m00,
            m01,
            m02,
            m03,
            m04,
            m05,
            m06,
            m07,
            m08,
            m09,
            m10,
            m11,
            m12,
            m13,
            m14,
            m15,
        ] = BINARY_MAGIC;
        let [v00, v01] = IDENTITY_VERSION.to_be_bytes();
        let [a00] = [HASH_ALGORITHM];
        let [l00, l01, l02, l03, l04, l05, l06, l07] = self.logical_length().get().to_be_bytes();
        let [
            d00,
            d01,
            d02,
            d03,
            d04,
            d05,
            d06,
            d07,
            d08,
            d09,
            d10,
            d11,
            d12,
            d13,
            d14,
            d15,
            d16,
            d17,
            d18,
            d19,
            d20,
            d21,
            d22,
            d23,
            d24,
            d25,
            d26,
            d27,
            d28,
            d29,
            d30,
            d31,
        ] = *self.digest();
        [
            m00, m01, m02, m03, m04, m05, m06, m07, m08, m09, m10, m11, m12, m13, m14, m15, v00,
            v01, a00, l00, l01, l02, l03, l04, l05, l06, l07, d00, d01, d02, d03, d04, d05, d06,
            d07, d08, d09, d10, d11, d12, d13, d14, d15, d16, d17, d18, d19, d20, d21, d22, d23,
            d24, d25, d26, d27, d28, d29, d30, d31,
        ]
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
