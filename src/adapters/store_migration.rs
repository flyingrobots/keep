//! Canonical version-2 store migration record adapters.

mod admitted_format_marker;
mod admitted_migration_intent;
mod admitted_migration_receipt;
mod canonical_format_marker;
mod empty_disposition_set_digest;
mod format_definition_digest;
mod format_marker_decode_error;
mod format_marker_decode_error_display;
mod format_marker_decoder;
mod format_marker_digest;
mod format_marker_encoder;
mod immutable_pool_inventory_digest;
mod initial_gc_state_digest;
mod initial_retention_state_digest;
mod migration_intent_decode_error;
mod migration_intent_decode_error_display;
mod migration_intent_decoder;
mod migration_intent_digest;
mod migration_phase;
mod migration_receipt_decode_error;
mod migration_receipt_decode_error_display;
mod migration_receipt_decoder;
mod migration_receipt_initial_state;
mod migration_record_bytes;
mod migration_synchronization_mask;
mod store_identifier;
mod store_root_identity;

pub use admitted_format_marker::AdmittedStoreFormatMarker;
pub use admitted_migration_intent::AdmittedStoreMigrationIntent;
pub use admitted_migration_receipt::AdmittedStoreMigrationReceipt;
pub use canonical_format_marker::CanonicalStoreFormatMarker;
pub use empty_disposition_set_digest::EmptyDispositionSetDigest;
pub use format_definition_digest::StoreFormatDefinitionDigest;
pub use format_marker_decode_error::StoreFormatMarkerDecodeError;
pub use format_marker_digest::StoreFormatMarkerDigest;
pub use immutable_pool_inventory_digest::ImmutablePoolInventoryDigest;
pub use initial_gc_state_digest::InitialGcStateDigest;
pub use initial_retention_state_digest::InitialRetentionStateDigest;
pub use migration_intent_decode_error::StoreMigrationIntentDecodeError;
pub use migration_intent_digest::StoreMigrationIntentDigest;
pub use migration_phase::StoreMigrationPhase;
pub use migration_receipt_decode_error::StoreMigrationReceiptDecodeError;
pub use migration_synchronization_mask::MigrationSynchronizationMask;
pub use store_identifier::StoreIdentifier;
pub use store_root_identity::{
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
};
