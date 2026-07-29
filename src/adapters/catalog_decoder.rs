//! Canonical catalog header and admission pipeline.

use super::{
    CatalogDecodeError, ChecksummedCatalog, catalog_entry_sequence, catalog_header_decoder,
    catalog_integrity, checksummed_catalog::CatalogMetadata,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

pub(super) const MAGIC: [u8; 16] = *b"KEEP:CATALOG:V1\0";
pub(super) const VERSION: u16 = 1;
pub(super) const FLAGS: u16 = 0;
pub(super) const MAXIMUM_ENTRY_COUNT: u64 = 1_048_576;
pub(super) const ALGORITHM: u8 = 1;

pub(super) fn decode(encoded: &[u8]) -> Result<ChecksummedCatalog<'_>, CatalogDecodeError> {
    let fields = catalog_header_decoder::decode(encoded)?;
    let metadata = validate_header(&fields)?;
    validate_observed_length(encoded, metadata.length())?;
    catalog_entry_sequence::validate(encoded, metadata.entry_count())?;
    let digest = catalog_integrity::validate(encoded)?;
    Ok(ChecksummedCatalog::from_verified_parts(
        encoded,
        metadata,
        CatalogDigest::from_validated(digest),
    ))
}

fn validate_header(
    fields: &catalog_header_decoder::DecodedCatalogHeader,
) -> Result<CatalogMetadata, CatalogDecodeError> {
    validate_fixed_fields(fields)?;
    let generation = CatalogGeneration::new(fields.generation)
        .map_err(|source| CatalogDecodeError::Generation { source })?;
    let predecessor = validate_predecessor(generation, fields.previous_digest)?;
    if fields.entry_count > MAXIMUM_ENTRY_COUNT {
        return Err(CatalogDecodeError::EntryCountOutOfBounds {
            maximum: MAXIMUM_ENTRY_COUNT,
            observed: fields.entry_count,
        });
    }
    let catalog_length = CatalogLength::new(fields.catalog_length)
        .map_err(|source| CatalogDecodeError::CatalogLength { source })?;
    validate_count_length(fields.entry_count, catalog_length)?;
    Ok(CatalogMetadata::new(
        generation,
        predecessor,
        fields.entry_count,
        catalog_length,
    ))
}

fn validate_fixed_fields(
    fields: &catalog_header_decoder::DecodedCatalogHeader,
) -> Result<(), CatalogDecodeError> {
    require_eq(fields.magic, MAGIC, |observed| {
        CatalogDecodeError::InvalidMagic { observed }
    })?;
    require_eq(fields.version, VERSION, |observed| {
        CatalogDecodeError::UnsupportedVersion {
            expected: VERSION,
            observed,
        }
    })?;
    require_eq(fields.flags, FLAGS, |observed| CatalogDecodeError::Flags {
        expected: FLAGS,
        observed,
    })?;
    require_eq(
        fields.header_length,
        catalog_header_decoder::HEADER_LENGTH,
        |observed| CatalogDecodeError::HeaderLength {
            expected: catalog_header_decoder::HEADER_LENGTH,
            observed,
        },
    )?;
    require_eq(
        fields.entry_length,
        catalog_header_decoder::ENTRY_LENGTH,
        |observed| CatalogDecodeError::EntryLength {
            expected: catalog_header_decoder::ENTRY_LENGTH,
            observed,
        },
    )?;
    require_eq(fields.checksum_algorithm, ALGORITHM, |observed| {
        CatalogDecodeError::ChecksumAlgorithm {
            expected: ALGORITHM,
            observed,
        }
    })?;
    require_eq(fields.digest_algorithm, ALGORITHM, |observed| {
        CatalogDecodeError::DigestAlgorithm {
            expected: ALGORITHM,
            observed,
        }
    })?;
    let expected = [0_u8; 46];
    require_eq(fields.reserved, expected, |observed| {
        CatalogDecodeError::Reserved { expected, observed }
    })
}

fn validate_predecessor(
    generation: CatalogGeneration,
    observed: [u8; 32],
) -> Result<Option<CatalogDigest>, CatalogDecodeError> {
    let zero = [0_u8; 32];
    if generation.get() == 1 {
        return if observed == zero {
            Ok(None)
        } else {
            Err(CatalogDecodeError::UnexpectedPredecessor {
                generation: generation.get(),
                observed,
            })
        };
    }
    if observed == zero {
        return Err(CatalogDecodeError::MissingPredecessor {
            generation: generation.get(),
        });
    }
    Ok(Some(CatalogDigest::from_validated(observed)))
}

fn validate_count_length(
    entry_count: u64,
    observed: CatalogLength,
) -> Result<(), CatalogDecodeError> {
    let expected = entry_count
        .checked_mul(u64::from(catalog_header_decoder::ENTRY_LENGTH))
        .and_then(|bytes| bytes.checked_add(u64::from(catalog_header_decoder::HEADER_LENGTH)))
        .and_then(|bytes| {
            bytes.checked_add(u64::try_from(catalog_header_decoder::TRAILER_LENGTH).ok()?)
        })
        .ok_or(CatalogDecodeError::LengthArithmetic { entry_count })?;
    if expected == observed.get() {
        Ok(())
    } else {
        Err(CatalogDecodeError::EntryCountLengthMismatch {
            entry_count,
            expected,
            observed: observed.get(),
        })
    }
}

fn validate_observed_length(
    encoded: &[u8],
    declared: CatalogLength,
) -> Result<(), CatalogDecodeError> {
    let expected =
        usize::try_from(declared.get()).map_err(|_source| CatalogDecodeError::ObservedLength {
            declared: declared.get(),
            observed: encoded.len(),
        })?;
    if encoded.len() == expected {
        Ok(())
    } else {
        Err(CatalogDecodeError::ObservedLength {
            declared: declared.get(),
            observed: encoded.len(),
        })
    }
}

fn require_eq<T: Copy + Eq>(
    observed: T,
    expected: T,
    error: impl FnOnce(T) -> CatalogDecodeError,
) -> Result<(), CatalogDecodeError> {
    if observed == expected {
        Ok(())
    } else {
        Err(error(observed))
    }
}
