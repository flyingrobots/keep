//! Catalog-entry field and location-coordinate laws.

use std::error::Error;

use keep::{CatalogDecodeError, CatalogEntryDecodeError};

use super::{
    ENTRY_FLAGS_OFFSET, ENTRY_IDENTITY_LENGTH_OFFSET, ENTRY_PAYLOAD_LENGTH_OFFSET,
    ENTRY_RECORD_LENGTH_OFFSET, ENTRY_RECORD_OFFSET, ENTRY_RESERVED_OFFSET, FIRST_ENTRY_OFFSET,
    mutation_support,
};

#[test]
fn catalog_refuses_noncanonical_entry_fields_and_lengths() -> Result<(), Box<dyn Error>> {
    mutation_support::assert_byte_refusal(FIRST_ENTRY_OFFSET, 3, |error| {
        error
            == CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::UnknownRecordKind { observed: 3 },
            }
    })?;
    mutation_support::assert_byte_refusal(ENTRY_FLAGS_OFFSET, 1, |error| {
        error
            == CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::Flags {
                    expected: 0,
                    observed: 1,
                },
            }
    })?;
    mutation_support::assert_u16_refusal(ENTRY_IDENTITY_LENGTH_OFFSET, 35, |error| {
        error
            == CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::IdentityLength {
                    record_kind: 1,
                    expected: 36,
                    observed: 35,
                },
            }
    })?;
    mutation_support::assert_u64_refusal(ENTRY_RECORD_OFFSET, 63, |error| {
        error
            == CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::RecordOffset {
                    minimum: 64,
                    observed: 63,
                },
            }
    })?;
    mutation_support::assert_u64_refusal(ENTRY_RECORD_LENGTH_OFFSET, 144, |error| {
        error
            == CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::RecordLengthMismatch {
                    payload_length: 1,
                    expected: 145,
                    observed: 144,
                },
            }
    })?;
    mutation_support::assert_u64_refusal(ENTRY_PAYLOAD_LENGTH_OFFSET, 2, |error| {
        matches!(
            error,
            CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::ChunkPayloadLengthMismatch {
                    identity_length: 1,
                    payload_length: 2,
                },
            }
        )
    })?;
    mutation_support::assert_byte_refusal(ENTRY_RESERVED_OFFSET, 1, |error| {
        matches!(
            error,
            CatalogDecodeError::Entry {
                index: 0,
                source: CatalogEntryDecodeError::Reserved { .. },
            }
        )
    })
}
