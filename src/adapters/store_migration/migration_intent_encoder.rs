//! This boundary module owns canonical migration-intent encoding.

use super::admitted_migration_intent::StoreMigrationIntentFields;
use super::migration_intent_format::StoreIdentifierFields;
use super::{
    CanonicalStoreMigrationIntent, ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest,
    StoreIdentifier, StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
    migration_intent_format as format,
};
use crate::CatalogSnapshot;

#[derive(Clone, Copy)]
struct RootIdentities {
    device: StoreRootDeviceIdentity,
    mount: StoreRootMountIdentity,
    file: StoreRootFileIdentity,
}

pub(super) fn encode(
    snapshot: &CatalogSnapshot<'_, '_, '_>,
    inventory_digest: ImmutablePoolInventoryDigest,
    root_device_identity: StoreRootDeviceIdentity,
    root_mount_identity: StoreRootMountIdentity,
    root_file_identity: StoreRootFileIdentity,
) -> CanonicalStoreMigrationIntent {
    let fields = StoreIdentifierFields {
        catalog_generation: snapshot.generation(),
        catalog_length: snapshot.catalog_length(),
        catalog_digest: snapshot.catalog_digest(),
        predecessor_catalog_digest: snapshot.previous_catalog_digest(),
        inventory_digest,
        target_definition_digest: StoreFormatDefinitionDigest::VERSION_TWO,
    };
    let roots = RootIdentities {
        device: root_device_identity,
        mount: root_mount_identity,
        file: root_file_identity,
    };
    let store_identifier = format::store_identifier(&fields);
    let mut encoded = [0_u8; format::ENCODED_LENGTH];
    let (preimage, checksum_slot) = encoded.split_at_mut(format::CHECKSUM_OFFSET);
    write_preimage(preimage, &fields, roots, store_identifier);
    checksum_slot.copy_from_slice(&format::checksum(preimage));
    let digest = format::digest(&encoded);
    CanonicalStoreMigrationIntent::admitted(
        encoded,
        StoreMigrationIntentFields {
            catalog_generation: fields.catalog_generation,
            catalog_length: fields.catalog_length,
            catalog_digest: fields.catalog_digest,
            predecessor_catalog_digest: fields.predecessor_catalog_digest,
            inventory_digest: fields.inventory_digest,
            root_device_identity: roots.device,
            root_mount_identity: roots.mount,
            root_file_identity: roots.file,
            target_definition_digest: fields.target_definition_digest,
            store_identifier,
        },
        digest,
    )
}

fn write_preimage(
    output: &mut [u8],
    fields: &StoreIdentifierFields,
    roots: RootIdentities,
    store_identifier: StoreIdentifier,
) {
    let (magic, output) = output.split_at_mut(16);
    magic.copy_from_slice(&format::MAGIC);
    let (version, output) = output.split_at_mut(2);
    version.copy_from_slice(&format::VERSION.to_be_bytes());
    let (record_length, output) = output.split_at_mut(2);
    record_length.copy_from_slice(&format::RECORD_LENGTH.to_be_bytes());
    let (flags, output) = output.split_at_mut(4);
    flags.copy_from_slice(&0_u32.to_be_bytes());
    let (generation, output) = output.split_at_mut(8);
    generation.copy_from_slice(&fields.catalog_generation.get().to_be_bytes());
    let (catalog_length, output) = output.split_at_mut(8);
    catalog_length.copy_from_slice(&fields.catalog_length.get().to_be_bytes());
    let (catalog_digest, output) = output.split_at_mut(32);
    catalog_digest.copy_from_slice(fields.catalog_digest.as_bytes());
    let predecessor = fields
        .predecessor_catalog_digest
        .as_ref()
        .map_or(&format::ZERO_DIGEST, crate::CatalogDigest::as_bytes);
    let (predecessor_digest, output) = output.split_at_mut(32);
    predecessor_digest.copy_from_slice(predecessor);
    let (inventory_digest, output) = output.split_at_mut(32);
    inventory_digest.copy_from_slice(fields.inventory_digest.as_bytes());
    let (device_identity, output) = output.split_at_mut(8);
    device_identity.copy_from_slice(&roots.device.get().to_be_bytes());
    let (mount_identity, output) = output.split_at_mut(8);
    mount_identity.copy_from_slice(&roots.mount.get().to_be_bytes());
    let (file_identity, output) = output.split_at_mut(8);
    file_identity.copy_from_slice(&roots.file.get().to_be_bytes());
    let (definition_digest, store_identifier_slot) = output.split_at_mut(32);
    definition_digest.copy_from_slice(fields.target_definition_digest.as_bytes());
    store_identifier_slot.copy_from_slice(store_identifier.as_bytes());
}
