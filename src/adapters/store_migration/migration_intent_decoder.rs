//! This boundary module owns store-migration intent decoding order.

use super::admitted_migration_intent::StoreMigrationIntentFields;
use super::migration_record_bytes::{
    read_array, read_u16, read_u32, read_u64, require_length, wrong_length,
};
use super::{
    AdmittedStoreMigrationIntent, ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest,
    StoreIdentifier, StoreMigrationIntentDecodeError, StoreMigrationIntentDigest,
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

const CHECKSUM_OFFSET: usize = 224;
const MAGIC: [u8; 16] = *b"KEEP:MIG:INT2\0\0\0";
const VERSION: u16 = 2;
const RECORD_LENGTH: u16 = 256;
const CHECKSUM_DOMAIN: &[u8] = b"keep.store-migration-intent-checksum/v2\0";
const DIGEST_DOMAIN: &[u8] = b"keep.store-migration-intent/v2\0";
const STORE_IDENTIFIER_DOMAIN: &[u8] = b"keep.store-identifier/v2\0";
const ZERO_DIGEST: [u8; 32] = [0; 32];

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
        digest(encoded),
    ))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), StoreMigrationIntentDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != MAGIC {
        return Err(StoreMigrationIntentDecodeError::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != VERSION {
        return Err(StoreMigrationIntentDecodeError::UnsupportedVersion {
            expected: VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != RECORD_LENGTH {
        return Err(StoreMigrationIntentDecodeError::InvalidRecordLength {
            expected: RECORD_LENGTH,
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
        .get(..CHECKSUM_OFFSET)
        .ok_or_else(|| wrong_length(encoded))?;
    let observed = read_array(encoded, CHECKSUM_OFFSET)?;
    let expected = hash(CHECKSUM_DOMAIN, &[preimage]);
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
        return if observed == ZERO_DIGEST {
            Ok(None)
        } else {
            Err(StoreMigrationIntentDecodeError::NonZeroInitialPredecessor { observed })
        };
    }
    if observed == ZERO_DIGEST {
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
    let predecessor = fields
        .predecessor_catalog_digest
        .as_ref()
        .map_or(&ZERO_DIGEST, CatalogDigest::as_bytes);
    let expected = hash(
        STORE_IDENTIFIER_DOMAIN,
        &[
            &fields.catalog_generation.get().to_be_bytes(),
            &fields.catalog_length.get().to_be_bytes(),
            fields.catalog_digest.as_bytes(),
            predecessor,
            fields.inventory_digest.as_bytes(),
            fields.target_definition_digest.as_bytes(),
        ],
    );
    let observed = *fields.store_identifier.as_bytes();
    if observed == expected {
        Ok(())
    } else {
        Err(StoreMigrationIntentDecodeError::StoreIdentifierMismatch { expected, observed })
    }
}

fn digest(encoded: &[u8]) -> StoreMigrationIntentDigest {
    StoreMigrationIntentDigest::from_hash(hash(DIGEST_DOMAIN, &[encoded]))
}

fn hash(domain: &[u8], fields: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        hasher.update(field);
    }
    *hasher.finalize().as_bytes()
}
