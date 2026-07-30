//! This boundary module owns admitted store-migration intent evidence.

use super::{
    ImmutablePoolInventoryDigest, StoreFormatDefinitionDigest, StoreIdentifier,
    StoreMigrationIntentDecodeError, StoreMigrationIntentDigest, StoreRootDeviceIdentity,
    StoreRootFileIdentity, StoreRootMountIdentity, migration_intent_decoder,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Semantic fields admitted from one canonical migration intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoreMigrationIntentFields {
    pub(super) catalog_generation: CatalogGeneration,
    pub(super) catalog_length: CatalogLength,
    pub(super) catalog_digest: CatalogDigest,
    pub(super) predecessor_catalog_digest: Option<CatalogDigest>,
    pub(super) inventory_digest: ImmutablePoolInventoryDigest,
    pub(super) root_device_identity: StoreRootDeviceIdentity,
    pub(super) root_mount_identity: StoreRootMountIdentity,
    pub(super) root_file_identity: StoreRootFileIdentity,
    pub(super) target_definition_digest: StoreFormatDefinitionDigest,
    pub(super) store_identifier: StoreIdentifier,
}

/// Borrowed canonical version-2 store-migration intent.
///
/// Admission proves record framing, integrity, internal generation laws, the
/// registered target definition, and deterministic store identity. It does not
/// prove that the named catalog, inventory, or physical root is current.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedStoreMigrationIntent<'encoded> {
    encoded: &'encoded [u8],
    fields: StoreMigrationIntentFields,
    digest: StoreMigrationIntentDigest,
}

impl<'encoded> AdmittedStoreMigrationIntent<'encoded> {
    /// Decodes and admits one exact canonical migration intent.
    ///
    /// # Errors
    ///
    /// Returns [`StoreMigrationIntentDecodeError`] for invalid framing,
    /// integrity, catalog coordinates, predecessor state, definition identity,
    /// or store identity.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, StoreMigrationIntentDecodeError> {
        migration_intent_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the positive catalog generation named by the intent.
    pub const fn catalog_generation(&self) -> CatalogGeneration {
        self.fields.catalog_generation
    }

    /// Returns the exact admitted catalog byte length.
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

    /// Returns the serialized root device coordinate named by the intent.
    pub const fn root_device_identity(&self) -> StoreRootDeviceIdentity {
        self.fields.root_device_identity
    }

    /// Returns the serialized root mount coordinate named by the intent.
    pub const fn root_mount_identity(&self) -> StoreRootMountIdentity {
        self.fields.root_mount_identity
    }

    /// Returns the serialized root file coordinate named by the intent.
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

    /// Returns the domain-separated identity of all intent bytes.
    pub const fn digest(&self) -> StoreMigrationIntentDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
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
