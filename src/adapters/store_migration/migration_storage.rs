//! This boundary module owns blocking store-migration durability capabilities.

use std::io;

use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
};

/// Blocking storage capabilities for one writer-locked version-2 migration.
///
/// An implementation must retain exclusive writer authority and one pinned
/// store root for the complete operation. After `verify_current`, each method
/// corresponds to one [`StoreMigrationPhase`](super::StoreMigrationPhase) and
/// must not report success before its durability and verification obligations
/// are complete.
pub trait StoreMigrationStorage {
    /// Revalidates the exact version-1 authority bound by `intent`.
    ///
    /// This must verify the catalog coordinates, inventory, physical root,
    /// version-1 format, and absence of migration or version-2 artifacts.
    ///
    /// # Errors
    ///
    /// Returns the exact current-state or recovery-required refusal.
    fn verify_current(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()>;

    /// Exclusively creates and completely writes `migration.intent.next`.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_intent_stage(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()>;

    /// Synchronizes the complete intent stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_intent_stage(&mut self) -> io::Result<()>;

    /// Links and exactly verifies canonical `migration.intent`.
    ///
    /// # Errors
    ///
    /// Returns the exact link, reopen, or verification failure.
    fn link_intent(&mut self, intent: &CanonicalStoreMigrationIntent) -> io::Result<()>;

    /// Synchronizes the store root after the intent link.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_intent(&mut self) -> io::Result<()>;

    /// Removes only the retained intent stage.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_intent_stage(&mut self) -> io::Result<()>;

    /// Synchronizes the store root after intent-stage cleanup.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_intent_cleanup(&mut self) -> io::Result<()>;

    /// Creates or exactly admits persistent `reader.lock`.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, open, or verification failure.
    fn admit_reader_fence(&mut self) -> io::Result<()>;

    /// Creates or exactly admits the complete version-2 directory prefix.
    ///
    /// # Errors
    ///
    /// Returns the exact namespace creation or admission failure.
    fn admit_namespace_prefix(&mut self) -> io::Result<()>;

    /// Synchronizes created namespaces and the store root.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_namespace(&mut self) -> io::Result<()>;

    /// Exclusively creates and completely writes `FORMAT.next`.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_marker_stage(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()>;

    /// Synchronizes the complete marker stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_marker_stage(&mut self) -> io::Result<()>;

    /// Links and exactly verifies canonical `FORMAT`.
    ///
    /// # Errors
    ///
    /// Returns the exact link, reopen, or verification failure.
    fn link_marker(&mut self, marker: &CanonicalStoreFormatMarker) -> io::Result<()>;

    /// Synchronizes the store root after the marker link.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_marker(&mut self) -> io::Result<()>;

    /// Removes only the retained marker stage.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_marker_stage(&mut self) -> io::Result<()>;

    /// Synchronizes the store root after marker-stage cleanup.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_marker_cleanup(&mut self) -> io::Result<()>;

    /// Exclusively creates and completely writes `migration.receipt.next`.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_receipt_stage(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()>;

    /// Synchronizes the complete receipt stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_receipt_stage(&mut self) -> io::Result<()>;

    /// Links and exactly verifies canonical `migration.receipt`.
    ///
    /// # Errors
    ///
    /// Returns the exact link, reopen, or verification failure.
    fn link_receipt(&mut self, receipt: &CanonicalStoreMigrationReceipt) -> io::Result<()>;

    /// Synchronizes the store root after the receipt link.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_receipt(&mut self) -> io::Result<()>;

    /// Removes only the retained receipt stage.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_receipt_stage(&mut self) -> io::Result<()>;

    /// Synchronizes the store root after receipt-stage cleanup.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_after_receipt_cleanup(&mut self) -> io::Result<()>;
}
