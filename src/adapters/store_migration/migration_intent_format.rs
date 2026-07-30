//! This boundary module owns shared migration-intent format identity.

use super::{
    ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest, StoreIdentifier,
    StoreMigrationIntentDigest,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

pub(super) const CHECKSUM_OFFSET: usize = 224;
pub(super) const ENCODED_LENGTH: usize = 256;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:MIG:INT2\0\0\0";
pub(super) const RECORD_LENGTH: u16 = 256;
pub(super) const VERSION: u16 = 2;
const CHECKSUM_DOMAIN: &[u8] = b"keep.store-migration-intent-checksum/v2\0";
const DIGEST_DOMAIN: &[u8] = b"keep.store-migration-intent/v2\0";
const STORE_IDENTIFIER_DOMAIN: &[u8] = b"keep.store-identifier/v2\0";
pub(super) const ZERO_DIGEST: [u8; 32] = [0; 32];

pub(super) struct StoreIdentifierFields {
    pub(super) catalog_generation: CatalogGeneration,
    pub(super) catalog_length: CatalogLength,
    pub(super) catalog_digest: CatalogDigest,
    pub(super) predecessor_catalog_digest: Option<CatalogDigest>,
    pub(super) inventory_digest: ImmutablePoolInventoryDigest,
    pub(super) target_definition_digest: StoreFormatDefinitionDigest,
}

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    hash(CHECKSUM_DOMAIN, &[preimage])
}

pub(super) fn digest(encoded: &[u8]) -> StoreMigrationIntentDigest {
    StoreMigrationIntentDigest::from_hash(hash(DIGEST_DOMAIN, &[encoded]))
}

pub(super) fn store_identifier(fields: &StoreIdentifierFields) -> StoreIdentifier {
    let predecessor = fields
        .predecessor_catalog_digest
        .as_ref()
        .map_or(&ZERO_DIGEST, CatalogDigest::as_bytes);
    StoreIdentifier::from_hash(hash(
        STORE_IDENTIFIER_DOMAIN,
        &[
            &fields.catalog_generation.get().to_be_bytes(),
            &fields.catalog_length.get().to_be_bytes(),
            fields.catalog_digest.as_bytes(),
            predecessor,
            fields.inventory_digest.as_bytes(),
            fields.target_definition_digest.as_bytes(),
        ],
    ))
}

fn hash(domain: &[u8], fields: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        hasher.update(field);
    }
    *hasher.finalize().as_bytes()
}
