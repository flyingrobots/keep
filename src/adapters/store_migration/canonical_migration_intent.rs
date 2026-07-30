//! This boundary module owns canonical owned migration-intent bytes.

use super::admitted_migration_intent::StoreMigrationIntentFields;
use super::{
    ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest, StoreIdentifier,
    StoreMigrationIntentDigest, StoreRootDeviceIdentity, StoreRootFileIdentity,
    StoreRootMountIdentity, migration_intent_encoder, migration_intent_format,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength, CatalogSnapshot};

/// Owned canonical version-2 store-migration intent.
///
/// Construction preserves admitted catalog coordinates and serializes the
/// supplied inventory and physical-root coordinates. It does not prove that
/// the inventory or physical root remains current.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalStoreMigrationIntent {
    encoded: [u8; migration_intent_format::ENCODED_LENGTH],
    fields: StoreMigrationIntentFields,
    digest: StoreMigrationIntentDigest,
}

impl CanonicalStoreMigrationIntent {
    /// Owns the exact bytes and identities of an admitted intent.
    pub const fn from_admitted(intent: &super::AdmittedStoreMigrationIntent<'_>) -> Self {
        let mut encoded = [0_u8; migration_intent_format::ENCODED_LENGTH];
        encoded.copy_from_slice(intent.encoded());
        Self {
            encoded,
            fields: StoreMigrationIntentFields {
                catalog_generation: intent.catalog_generation(),
                catalog_length: intent.catalog_length(),
                catalog_digest: intent.catalog_digest(),
                predecessor_catalog_digest: intent.predecessor_catalog_digest(),
                inventory_digest: intent.inventory_digest(),
                root_device_identity: intent.root_device_identity(),
                root_mount_identity: intent.root_mount_identity(),
                root_file_identity: intent.root_file_identity(),
                target_definition_digest: intent.target_definition_digest(),
                store_identifier: intent.store_identifier(),
            },
            digest: intent.digest(),
        }
    }

    /// Constructs one canonical intent from typed migration coordinates.
    pub fn from_snapshot(
        snapshot: &CatalogSnapshot<'_, '_, '_>,
        inventory_digest: ImmutablePoolInventoryDigest,
        root_device_identity: StoreRootDeviceIdentity,
        root_mount_identity: StoreRootMountIdentity,
        root_file_identity: StoreRootFileIdentity,
    ) -> Self {
        migration_intent_encoder::encode(
            snapshot,
            inventory_digest,
            root_device_identity,
            root_mount_identity,
            root_file_identity,
        )
    }

    /// Returns the exact canonical intent bytes.
    pub const fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the domain-separated identity of all intent bytes.
    pub const fn digest(&self) -> StoreMigrationIntentDigest {
        self.digest
    }

    /// Returns the positive catalog generation named by the intent.
    pub const fn catalog_generation(&self) -> CatalogGeneration {
        self.fields.catalog_generation
    }

    /// Returns the exact catalog byte length named by the intent.
    pub const fn catalog_length(&self) -> CatalogLength {
        self.fields.catalog_length
    }

    /// Returns the catalog digest named by the intent.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.fields.catalog_digest
    }

    /// Returns the generation-relative predecessor digest.
    pub const fn predecessor_catalog_digest(&self) -> Option<CatalogDigest> {
        self.fields.predecessor_catalog_digest
    }

    /// Returns the immutable-pool inventory digest named by the intent.
    pub const fn inventory_digest(&self) -> ImmutablePoolInventoryDigest {
        self.fields.inventory_digest
    }

    /// Returns the serialized root device coordinate.
    pub const fn root_device_identity(&self) -> StoreRootDeviceIdentity {
        self.fields.root_device_identity
    }

    /// Returns the serialized root mount coordinate.
    pub const fn root_mount_identity(&self) -> StoreRootMountIdentity {
        self.fields.root_mount_identity
    }

    /// Returns the serialized root file coordinate.
    pub const fn root_file_identity(&self) -> StoreRootFileIdentity {
        self.fields.root_file_identity
    }

    /// Returns the registered target format-definition digest.
    pub const fn target_definition_digest(&self) -> StoreFormatDefinitionDigest {
        self.fields.target_definition_digest
    }

    /// Returns the deterministic logical store identity.
    pub const fn store_identifier(&self) -> StoreIdentifier {
        self.fields.store_identifier
    }

    pub(super) const fn admitted(
        encoded: [u8; migration_intent_format::ENCODED_LENGTH],
        fields: StoreMigrationIntentFields,
        digest: StoreMigrationIntentDigest,
    ) -> Self {
        Self {
            encoded,
            fields,
            digest,
        }
    }
}
