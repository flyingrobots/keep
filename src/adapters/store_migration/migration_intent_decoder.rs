//! This boundary module owns store-migration intent decoding order.

use super::admitted_migration_intent::StoreMigrationIntentFields;
use super::migration_intent_format::{self as format, StoreIdentifierFields};
use super::migration_record_bytes::{
    read_array, read_u16, read_u32, read_u64, require_length, wrong_length,
};
use super::{
    AdmittedStoreMigrationIntent, ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest,
    StoreIdentifier, StoreMigrationIntentDecodeError, StoreRootDeviceIdentity,
    StoreRootFileIdentity, StoreRootMountIdentity,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

pub(super) fn decode(
    encoded: &[u8],
) -> Result<AdmittedStoreMigrationIntent<'_>, StoreMigrationIntentDecodeError> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let catalog_generation = read_catalog_generation(encoded)?;
    let catalog_length = read_catalog_length(encoded)?;
    let catalog_digest = CatalogDigest::from_validated(read_array(encoded, 40)?);
    let predecessor_catalog_digest =
        read_predecessor(catalog_generation, read_array(encoded, 72)?)?;
    let fields = StoreMigrationIntentFields {
        catalog_generation,
        catalog_length,
        catalog_digest,
        predecessor_catalog_digest,
        inventory_digest: ImmutablePoolInventoryDigest::from_admitted(read_array(encoded, 104)?),
        root_device_identity: StoreRootDeviceIdentity::from_admitted(read_u64(encoded, 136)?),
        root_mount_identity: StoreRootMountIdentity::from_admitted(read_u64(encoded, 144)?),
        root_file_identity: StoreRootFileIdentity::from_admitted(read_u64(encoded, 152)?),
        target_definition_digest: read_definition_digest(encoded)?,
        store_identifier: StoreIdentifier::from_hash(read_array(encoded, 192)?),
    };
    verify_store_identifier(&fields)?;
    Ok(AdmittedStoreMigrationIntent::admitted(
        encoded,
        fields,
        format::digest(encoded),
    ))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), StoreMigrationIntentDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != format::MAGIC {
        return Err(StoreMigrationIntentDecodeError::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != format::VERSION {
        return Err(StoreMigrationIntentDecodeError::UnsupportedVersion {
            expected: format::VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != format::RECORD_LENGTH {
        return Err(StoreMigrationIntentDecodeError::InvalidRecordLength {
            expected: format::RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(StoreMigrationIntentDecodeError::UnsupportedFlags { observed: flags });
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), StoreMigrationIntentDecodeError> {
    let preimage = encoded
        .get(..format::CHECKSUM_OFFSET)
        .ok_or_else(|| wrong_length(encoded))?;
    let observed = read_array(encoded, format::CHECKSUM_OFFSET)?;
    let expected = format::checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(StoreMigrationIntentDecodeError::ChecksumMismatch { expected, observed })
    }
}

fn read_catalog_generation(
    encoded: &[u8],
) -> Result<CatalogGeneration, StoreMigrationIntentDecodeError> {
    let observed = read_u64(encoded, 24)?;
    CatalogGeneration::new(observed).map_err(|source| {
        StoreMigrationIntentDecodeError::InvalidCatalogGeneration { observed, source }
    })
}

fn read_catalog_length(encoded: &[u8]) -> Result<CatalogLength, StoreMigrationIntentDecodeError> {
    let observed = read_u64(encoded, 32)?;
    CatalogLength::new(observed).map_err(|source| {
        StoreMigrationIntentDecodeError::InvalidCatalogLength { observed, source }
    })
}

fn read_predecessor(
    generation: CatalogGeneration,
    observed: [u8; 32],
) -> Result<Option<CatalogDigest>, StoreMigrationIntentDecodeError> {
    if generation.get() == 1 {
        return if observed == format::ZERO_DIGEST {
            Ok(None)
        } else {
            Err(StoreMigrationIntentDecodeError::NonZeroInitialPredecessor { observed })
        };
    }
    if observed == format::ZERO_DIGEST {
        return Err(
            StoreMigrationIntentDecodeError::MissingSuccessorPredecessor {
                generation: generation.get(),
            },
        );
    }
    Ok(Some(CatalogDigest::from_validated(observed)))
}

fn read_definition_digest(
    encoded: &[u8],
) -> Result<StoreFormatDefinitionDigest, StoreMigrationIntentDecodeError> {
    let observed = read_array(encoded, 160)?;
    if observed == *StoreFormatDefinitionDigest::VERSION_TWO.as_bytes() {
        Ok(StoreFormatDefinitionDigest::VERSION_TWO)
    } else {
        Err(StoreMigrationIntentDecodeError::DefinitionDigestMismatch {
            expected: *StoreFormatDefinitionDigest::VERSION_TWO.as_bytes(),
            observed,
        })
    }
}

fn verify_store_identifier(
    fields: &StoreMigrationIntentFields,
) -> Result<(), StoreMigrationIntentDecodeError> {
    let expected = format::store_identifier(&StoreIdentifierFields {
        catalog_generation: fields.catalog_generation,
        catalog_length: fields.catalog_length,
        catalog_digest: fields.catalog_digest,
        predecessor_catalog_digest: fields.predecessor_catalog_digest,
        inventory_digest: fields.inventory_digest,
        target_definition_digest: fields.target_definition_digest,
    });
    let observed = *fields.store_identifier.as_bytes();
    if observed == *expected.as_bytes() {
        Ok(())
    } else {
        Err(StoreMigrationIntentDecodeError::StoreIdentifierMismatch {
            expected: *expected.as_bytes(),
            observed,
        })
    }
}
