//! This boundary module owns admitted store-migration completion evidence.

use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, EmptyDispositionSetDigest,
    InitialGcStateDigest, InitialRetentionStateDigest, MigrationSynchronizationMask,
    StoreFormatMarkerDigest, StoreIdentifier, StoreMigrationIntentDigest,
    StoreMigrationReceiptDecodeError, migration_receipt_decoder,
};

/// Semantic fields admitted from one canonical migration receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoreMigrationReceiptFields {
    pub(super) intent_digest: StoreMigrationIntentDigest,
    pub(super) store_identifier: StoreIdentifier,
    pub(super) format_marker_digest: StoreFormatMarkerDigest,
    pub(super) initial_retention_state_digest: InitialRetentionStateDigest,
    pub(super) initial_gc_state_digest: InitialGcStateDigest,
    pub(super) empty_disposition_set_digest: EmptyDispositionSetDigest,
    pub(super) synchronization_mask: MigrationSynchronizationMask,
}

/// Borrowed canonical version-2 store-migration receipt.
///
/// Admission proves record integrity, exact binding to caller-supplied admitted
/// intent and marker evidence, registered empty-state digests, and the complete
/// synchronization mask. It does not prove that filesystem transitions
/// actually occurred; production recovery must establish that separately.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedStoreMigrationReceipt<'encoded> {
    encoded: &'encoded [u8],
    fields: StoreMigrationReceiptFields,
}

impl<'encoded> AdmittedStoreMigrationReceipt<'encoded> {
    /// Decodes and admits one exact receipt bound to `intent` and `marker`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreMigrationReceiptDecodeError`] for invalid framing,
    /// integrity, binding, registered state digest, or synchronization bits.
    pub fn decode(
        encoded: &'encoded [u8],
        intent: &AdmittedStoreMigrationIntent<'_>,
        marker: &AdmittedStoreFormatMarker<'_>,
    ) -> Result<Self, StoreMigrationReceiptDecodeError> {
        migration_receipt_decoder::decode(encoded, intent, marker)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the exact bound migration-intent digest.
    pub const fn intent_digest(&self) -> StoreMigrationIntentDigest {
        self.fields.intent_digest
    }

    /// Returns the exact bound logical store identity.
    pub const fn store_identifier(&self) -> StoreIdentifier {
        self.fields.store_identifier
    }

    /// Returns the exact bound format-marker digest.
    pub const fn format_marker_digest(&self) -> StoreFormatMarkerDigest {
        self.fields.format_marker_digest
    }

    /// Returns the registered empty retention-state digest.
    pub const fn initial_retention_state_digest(&self) -> InitialRetentionStateDigest {
        self.fields.initial_retention_state_digest
    }

    /// Returns the registered empty garbage-collection-state digest.
    pub const fn initial_gc_state_digest(&self) -> InitialGcStateDigest {
        self.fields.initial_gc_state_digest
    }

    /// Returns the registered empty recovery-disposition-set digest.
    pub const fn empty_disposition_set_digest(&self) -> EmptyDispositionSetDigest {
        self.fields.empty_disposition_set_digest
    }

    /// Returns the complete admitted synchronization mask.
    pub const fn synchronization_mask(&self) -> MigrationSynchronizationMask {
        self.fields.synchronization_mask
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
        fields: StoreMigrationReceiptFields,
    ) -> Self {
        Self { encoded, fields }
    }
}
