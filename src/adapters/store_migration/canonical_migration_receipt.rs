//! This boundary module owns canonical owned migration-receipt bytes.

use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, migration_receipt_encoder,
    migration_receipt_format,
};

/// Owned canonical version-2 store-migration completion receipt.
///
/// Construction binds canonical artifacts and the registered complete initial
/// state. It does not prove that the named filesystem transitions occurred.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalStoreMigrationReceipt {
    encoded: [u8; migration_receipt_format::ENCODED_LENGTH],
}

impl CanonicalStoreMigrationReceipt {
    /// Constructs the one complete receipt for `intent` and `marker`.
    pub fn from_canonical(
        intent: &CanonicalStoreMigrationIntent,
        marker: &CanonicalStoreFormatMarker,
    ) -> Self {
        migration_receipt_encoder::encode(intent, marker)
    }

    /// Returns the exact canonical receipt bytes.
    pub const fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    pub(super) const fn admitted(encoded: [u8; migration_receipt_format::ENCODED_LENGTH]) -> Self {
        Self { encoded }
    }
}
