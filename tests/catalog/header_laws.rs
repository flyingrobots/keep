//! Catalog-header canonicalization and predecessor laws.

use std::error::Error;

use keep::{CatalogDecodeError, CatalogGenerationError, CatalogLengthError};

use super::{
    CATALOG_LENGTH_OFFSET, CHECKSUM_ALGORITHM_OFFSET, DIGEST_ALGORITHM_OFFSET, ENTRY_COUNT_OFFSET,
    ENTRY_LENGTH_OFFSET, FLAGS_OFFSET, GENERATION_OFFSET, GENERATION_TWO_HEX, HEADER_LENGTH_OFFSET,
    HEADER_RESERVED_OFFSET, PREVIOUS_DIGEST_OFFSET, VERSION_OFFSET, catalog_bytes,
    mutation_support,
};

#[test]
fn catalog_refuses_noncanonical_header_fields() -> Result<(), Box<dyn Error>> {
    mutation_support::assert_byte_refusal(0, 0, |error| {
        matches!(error, CatalogDecodeError::InvalidMagic { .. })
    })?;
    mutation_support::assert_byte_refusal(VERSION_OFFSET + 1, 2, |error| {
        error
            == CatalogDecodeError::UnsupportedVersion {
                expected: 1,
                observed: 2,
            }
    })?;
    mutation_support::assert_u16_refusal(FLAGS_OFFSET, 1, |error| {
        error
            == CatalogDecodeError::Flags {
                expected: 0,
                observed: 1,
            }
    })?;
    mutation_support::assert_u16_refusal(HEADER_LENGTH_OFFSET, 127, |error| {
        error
            == CatalogDecodeError::HeaderLength {
                expected: 128,
                observed: 127,
            }
    })?;
    mutation_support::assert_u16_refusal(ENTRY_LENGTH_OFFSET, 159, |error| {
        error
            == CatalogDecodeError::EntryLength {
                expected: 160,
                observed: 159,
            }
    })?;
    mutation_support::assert_byte_refusal(CHECKSUM_ALGORITHM_OFFSET, 2, |error| {
        error
            == CatalogDecodeError::ChecksumAlgorithm {
                expected: 1,
                observed: 2,
            }
    })?;
    mutation_support::assert_byte_refusal(DIGEST_ALGORITHM_OFFSET, 2, |error| {
        error
            == CatalogDecodeError::DigestAlgorithm {
                expected: 1,
                observed: 2,
            }
    })?;
    mutation_support::assert_byte_refusal(HEADER_RESERVED_OFFSET, 1, |error| {
        matches!(error, CatalogDecodeError::Reserved { .. })
    })
}

#[test]
fn catalog_refuses_invalid_generation_predecessor_and_length() -> Result<(), Box<dyn Error>> {
    mutation_support::assert_u64_refusal(GENERATION_OFFSET, 0, |error| {
        error
            == CatalogDecodeError::Generation {
                source: CatalogGenerationError::Zero,
            }
    })?;
    mutation_support::assert_byte_refusal(PREVIOUS_DIGEST_OFFSET, 1, |error| {
        matches!(
            error,
            CatalogDecodeError::UnexpectedPredecessor { generation: 1, .. }
        )
    })?;
    mutation_support::assert_u64_refusal(ENTRY_COUNT_OFFSET, 1_048_577, |error| {
        error
            == CatalogDecodeError::EntryCountOutOfBounds {
                maximum: 1_048_576,
                observed: 1_048_577,
            }
    })?;
    mutation_support::assert_u64_refusal(CATALOG_LENGTH_OFFSET, 191, |error| {
        error
            == CatalogDecodeError::CatalogLength {
                source: CatalogLengthError::OutOfBounds {
                    minimum: 192,
                    maximum: 167_772_352,
                    observed: 191,
                },
            }
    })?;
    mutation_support::assert_u64_refusal(CATALOG_LENGTH_OFFSET, 512, |error| {
        error
            == CatalogDecodeError::EntryCountLengthMismatch {
                entry_count: 1,
                expected: 352,
                observed: 512,
            }
    })
}

#[test]
fn later_catalog_requires_one_nonzero_predecessor() -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(GENERATION_TWO_HEX)?;
    mutation_support::zero_range(&mut encoded, PREVIOUS_DIGEST_OFFSET, 32)?;
    mutation_support::assert_catalog_refusal(&mut encoded, |error| {
        matches!(
            error,
            CatalogDecodeError::MissingPredecessor { generation: 2 }
        )
    })
}
