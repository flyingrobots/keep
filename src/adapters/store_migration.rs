//! Canonical version-2 store migration record adapters.

mod admitted_format_marker;
mod admitted_migration_intent;
mod admitted_migration_receipt;
mod canonical_format_marker;
mod canonical_migration_intent;
mod canonical_migration_receipt;
mod empty_disposition_set_digest;
mod filesystem_inventory_catalog_errors;
mod filesystem_inventory_catalogs;
#[cfg(test)]
mod filesystem_inventory_catalogs_refusal_tests;
#[cfg(test)]
mod filesystem_inventory_catalogs_test_fixture;
#[cfg(test)]
mod filesystem_inventory_catalogs_tests;
mod filesystem_inventory_directory;
mod filesystem_inventory_error;
mod filesystem_inventory_error_display;
mod filesystem_inventory_file;
#[cfg(test)]
mod filesystem_inventory_file_tests;
mod filesystem_inventory_names;
#[cfg(test)]
mod filesystem_inventory_names_tests;
mod filesystem_inventory_reader;
#[cfg(test)]
mod filesystem_inventory_reader_tests;
mod filesystem_inventory_segments;
#[cfg(test)]
mod filesystem_inventory_segments_refusal_tests;
#[cfg(test)]
mod filesystem_inventory_segments_test_fixture;
#[cfg(test)]
mod filesystem_inventory_segments_tests;
mod format_definition_digest;
mod format_marker_decode_error;
mod format_marker_decode_error_display;
mod format_marker_decoder;
mod format_marker_digest;
mod format_marker_encoder;
mod immutable_pool_inventory_digest;
mod initial_gc_state_digest;
mod initial_retention_state_digest;
mod migration_catalog_admission;
mod migration_catalog_plan;
mod migration_catalog_records;
mod migration_error;
mod migration_execution;
mod migration_intent_decode_error;
mod migration_intent_decode_error_display;
mod migration_intent_decoder;
mod migration_intent_digest;
mod migration_intent_encoder;
mod migration_intent_format;
mod migration_inventory_entry;
mod migration_inventory_entry_count;
mod migration_inventory_entry_count_error;
mod migration_inventory_error;
mod migration_inventory_hasher;
mod migration_phase;
mod migration_receipt_decode_error;
mod migration_receipt_decode_error_display;
mod migration_receipt_decoder;
mod migration_receipt_encoder;
mod migration_receipt_format;
mod migration_receipt_initial_state;
mod migration_record_bytes;
mod migration_storage;
mod migration_synchronization_mask;
mod store_identifier;
mod store_root_identity;

pub use admitted_format_marker::AdmittedStoreFormatMarker;
pub use admitted_migration_intent::AdmittedStoreMigrationIntent;
pub use admitted_migration_receipt::AdmittedStoreMigrationReceipt;
pub use canonical_format_marker::CanonicalStoreFormatMarker;
pub use canonical_migration_intent::CanonicalStoreMigrationIntent;
pub use canonical_migration_receipt::CanonicalStoreMigrationReceipt;
pub use empty_disposition_set_digest::EmptyDispositionSetDigest;
pub use filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};
pub use filesystem_inventory_reader::FilesystemStoreMigrationInventoryReader;
pub use format_definition_digest::StoreFormatDefinitionDigest;
pub use format_marker_decode_error::StoreFormatMarkerDecodeError;
pub use format_marker_digest::StoreFormatMarkerDigest;
pub use immutable_pool_inventory_digest::ImmutablePoolInventoryDigest;
pub use initial_gc_state_digest::InitialGcStateDigest;
pub use initial_retention_state_digest::InitialRetentionStateDigest;
pub use migration_error::StoreMigrationError;
pub use migration_execution::execute_store_migration;
pub use migration_intent_decode_error::StoreMigrationIntentDecodeError;
pub use migration_intent_digest::StoreMigrationIntentDigest;
pub use migration_inventory_entry::StoreMigrationInventoryEntry;
pub use migration_inventory_entry_count::StoreMigrationInventoryEntryCount;
pub use migration_inventory_entry_count_error::StoreMigrationInventoryEntryCountError;
pub use migration_inventory_error::StoreMigrationInventoryError;
pub use migration_inventory_hasher::StoreMigrationInventoryHasher;
pub use migration_phase::StoreMigrationPhase;
pub use migration_receipt_decode_error::StoreMigrationReceiptDecodeError;
pub use migration_storage::StoreMigrationStorage;
pub use migration_synchronization_mask::MigrationSynchronizationMask;
pub use store_identifier::StoreIdentifier;
pub use store_root_identity::{
    StoreRootDeviceIdentity, StoreRootFileIdentity, StoreRootMountIdentity,
};
