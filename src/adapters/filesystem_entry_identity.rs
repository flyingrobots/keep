//! This module owns the device-and-inode evidence reported by identity refusals.

use super::filesystem_exact_record::EntryIdentity;

/// The device and inode a filesystem entry or open handle named when observed.
///
/// This is diagnostic evidence for one observation, not a stable store
/// identity: device and inode numbers are physical, host-local, and may be
/// reused after the entry is removed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FilesystemEntryIdentity {
    device: u64,
    inode: u64,
}

impl FilesystemEntryIdentity {
    pub(super) const fn new(device: u64, inode: u64) -> Self {
        Self { device, inode }
    }

    /// The observed device number.
    #[must_use]
    pub const fn device(self) -> u64 {
        self.device
    }

    /// The observed inode number.
    #[must_use]
    pub const fn inode(self) -> u64 {
        self.inode
    }
}

impl From<EntryIdentity> for FilesystemEntryIdentity {
    fn from(identity: EntryIdentity) -> Self {
        Self::new(identity.device(), identity.inode())
    }
}
