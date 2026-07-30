//! This boundary module owns canonical owned migration-intent bytes.

use super::{
    ImmutablePoolInventoryDigest, StoreIdentifier, StoreMigrationIntentDigest,
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
    migration_intent_encoder, migration_intent_format,
};
use crate::CatalogSnapshot;

/// Owned canonical version-2 store-migration intent.
///
/// Construction preserves admitted catalog coordinates and serializes the
/// supplied inventory and physical-root coordinates. It does not prove that
/// the inventory or physical root remains current.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalStoreMigrationIntent {
    encoded: [u8; migration_intent_format::ENCODED_LENGTH],
    digest: StoreMigrationIntentDigest,
    store_identifier: StoreIdentifier,
}

impl CanonicalStoreMigrationIntent {
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

    /// Returns the deterministic logical store identity.
    pub const fn store_identifier(&self) -> StoreIdentifier {
        self.store_identifier
    }

    pub(super) const fn admitted(
        encoded: [u8; migration_intent_format::ENCODED_LENGTH],
        digest: StoreMigrationIntentDigest,
        store_identifier: StoreIdentifier,
    ) -> Self {
        Self {
            encoded,
            digest,
            store_identifier,
        }
    }
}
