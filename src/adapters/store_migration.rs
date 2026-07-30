//! Canonical version-2 store migration record adapters.

mod admitted_format_marker;
mod admitted_migration_intent;
mod canonical_format_marker;
mod format_definition_digest;
mod format_marker_decode_error;
mod format_marker_decode_error_display;
mod format_marker_decoder;
mod format_marker_digest;
mod format_marker_encoder;
mod immutable_pool_inventory_digest;
mod migration_intent_bytes;
mod migration_intent_decode_error;
mod migration_intent_decode_error_display;
mod migration_intent_decoder;
mod migration_intent_digest;
mod store_identifier;
mod store_root_identity;

pub use admitted_format_marker::AdmittedStoreFormatMarker;
pub use admitted_migration_intent::AdmittedStoreMigrationIntent;
pub use canonical_format_marker::CanonicalStoreFormatMarker;
pub use format_definition_digest::StoreFormatDefinitionDigest;
pub use format_marker_decode_error::StoreFormatMarkerDecodeError;
pub use format_marker_digest::StoreFormatMarkerDigest;
pub use immutable_pool_inventory_digest::ImmutablePoolInventoryDigest;
pub use migration_intent_decode_error::StoreMigrationIntentDecodeError;
pub use migration_intent_digest::StoreMigrationIntentDigest;
pub use store_identifier::StoreIdentifier;
pub use store_root_identity::{
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
};
