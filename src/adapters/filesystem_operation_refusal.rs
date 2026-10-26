//! This module owns semantic refusals of local filesystem operations: writer
//! authority state, pinned-entry identity, recovery-inventory bounds,
//! crash-hook prefix bounds, and the admitted Linux ext4 platform profile.
//!
//! Each refusal travels as the payload of the `io::Error` its storage port
//! returns, so callers recover it with `io::Error::get_ref` and
//! `downcast_ref` instead of parsing a message. The `io::ErrorKind` chosen by
//! the constructing adapter is unchanged from the textual error it replaced.

use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

use super::{CatalogRestartArtifact, FilesystemEntryIdentity, FilesystemEntryKind};

#[cfg(test)]
#[path = "filesystem_operation_refusal_tests.rs"]
mod tests;

/// Why a local filesystem operation refused, distinct from its OS errors.
///
/// Evidence fields carry bounded coordinates and flags, never content bytes
/// or physical paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemOperationRefusal {
    /// The operation requires the writer authority this storage has not acquired.
    WriterAuthorityAbsent,
    /// Writer authority was already acquired by this storage.
    WriterAuthorityAlreadyAcquired,
    /// A pinned entry's name no longer refers to a directory.
    DirectoryRequired {
        /// Kind observed without following a symbolic link.
        observed: FilesystemEntryKind,
    },
    /// A pinned entry's name no longer identifies the pinned handle.
    IdentityChanged {
        /// Identity recorded when the entry was pinned.
        expected: FilesystemEntryIdentity,
        /// Identity the name currently resolves to.
        observed: FilesystemEntryIdentity,
    },
    /// A requested recovery-inventory count exceeds the protocol ceiling.
    InventoryLimit {
        /// The protocol ceiling.
        maximum: u64,
        /// The requested count.
        observed: u64,
    },
    /// Recovery-inventory count arithmetic would overflow.
    InventoryCountOverflow {
        /// The count that could not be incremented.
        observed: u64,
    },
    /// A recovery-inventory count does not fit this host's address space.
    InventoryAddressSpace {
        /// The requested count.
        observed: u64,
        /// The original checked conversion failure.
        source: TryFromIntError,
    },
    /// Recovery name capacity cannot include the drift witness.
    InventoryCapacityOverflow {
        /// The requested name count.
        observed: usize,
    },
    /// This platform cannot supply raw Unix recovery entry names.
    RawNamesUnsupported,
    /// A crash-hook prefix exceeds the encoded publication artifact.
    PrefixExceedsArtifact {
        /// The artifact being staged.
        artifact: CatalogRestartArtifact,
        /// The artifact's encoded length.
        maximum: usize,
        /// The requested prefix length.
        observed: usize,
    },
    /// The operation requires the admitted Linux ext4 platform profile.
    LinuxProfileRequired,
    /// An internal protocol directory name has no first component.
    EmptyProtocolName,
    /// `statx` omitted identity fields the profile requires.
    PlatformStatus {
        /// The required `statx` mask bits.
        required: u32,
        /// The returned `statx` mask bits.
        observed: u32,
    },
    /// A directory is not on a writable, case-sensitive ext4 filesystem.
    FilesystemProfile {
        /// The required ext4 superblock magic.
        expected_filesystem: i128,
        /// The observed filesystem type.
        observed_filesystem: i128,
        /// The observed `statvfs` mount flags.
        observed_mount_flags: u64,
        /// The observed inode flags.
        observed_inode_flags: u32,
    },
    /// A protocol directory crosses the root's device or mount.
    MountBoundaryChanged {
        /// The root's device number.
        expected_device: u64,
        /// The child's device number.
        observed_device: u64,
        /// The root's mount identity.
        expected_mount: u64,
        /// The child's mount identity.
        observed_mount: u64,
    },
}

impl fmt::Display for FilesystemOperationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WriterAuthorityAbsent => "operation requires retained writer authority",
            Self::WriterAuthorityAlreadyAcquired => "writer authority was already acquired",
            Self::DirectoryRequired { .. } => "pinned entry is no longer a directory",
            Self::IdentityChanged { .. } => "pinned entry changed identity",
            Self::InventoryLimit { .. } => "recovery count exceeds the protocol maximum",
            Self::InventoryCountOverflow { .. } => "recovery entry count overflowed",
            Self::InventoryAddressSpace { .. } => "recovery count does not fit the address space",
            Self::InventoryCapacityOverflow { .. } => "recovery name capacity overflowed",
            Self::RawNamesUnsupported => "raw recovery entry names require a Unix platform",
            Self::PrefixExceedsArtifact { .. } => "publication prefix exceeds the encoded artifact",
            Self::LinuxProfileRequired => "operation requires the admitted Linux ext4 profile",
            Self::EmptyProtocolName => "protocol directory name is empty",
            Self::PlatformStatus { .. } => "statx omitted required identity fields",
            Self::FilesystemProfile { .. } => {
                "directory is not on one local writable case-sensitive ext4 filesystem"
            }
            Self::MountBoundaryChanged { .. } => "protocol directory crosses the root's mount",
        })
    }
}

impl Error for FilesystemOperationRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InventoryAddressSpace { source, .. } => Some(source),
            _ => None,
        }
    }
}
