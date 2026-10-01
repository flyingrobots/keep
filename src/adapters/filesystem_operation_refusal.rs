//! This module owns semantic filesystem operation and platform-admission refusals.

use std::error::Error;
use std::fmt;

/// A filesystem operation's semantic refusal, distinct from its original OS errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemOperationRefusal {
    /// The root is not an admitted canonical initialization namespace.
    InitializationNamespace,
    /// The operation requires retained writer authority.
    WriterAuthorityAbsent,
    /// Writer authority has already been acquired.
    WriterAlreadyAcquired,
    /// A synchronization target is not a directory.
    DirectoryRequired,
    /// A named entry no longer identifies its pinned handle.
    IdentityChanged {
        /// The pinned device and inode.
        expected: (u64, u64),
        /// The current entry's device and inode.
        observed: (u64, u64),
    },
    /// A requested recovery inventory count exceeds the protocol ceiling.
    InventoryLimit {
        /// The protocol ceiling.
        maximum: u64,
        /// The requested count.
        observed: u64,
    },
    /// Recovery entry-count arithmetic overflowed.
    InventoryCountOverflow {
        /// The count before incrementing.
        observed: u64,
    },
    /// A recovery count exceeds the host address space.
    InventoryAddressSpace {
        /// The requested count.
        observed: u64,
        /// The original checked conversion failure.
        source: std::num::TryFromIntError,
    },
    /// Recovery name capacity cannot include the drift witness.
    InventoryCapacityOverflow {
        /// The requested name count.
        observed: usize,
    },
    /// The platform cannot supply the required raw Unix entry names.
    RawNamesUnsupported,
    /// A requested publication prefix exceeds the encoded record.
    PrefixBound {
        /// The record being staged.
        artifact: super::CatalogRestartArtifact,
        /// The encoded length.
        maximum: usize,
        /// The requested prefix length.
        observed: usize,
    },
    /// A recovery-stage read ended before its admitted boundary.
    IncompleteRead {
        /// The admitted byte length.
        expected: u64,
        /// The byte count actually read.
        observed: u64,
    },
    /// The operation requires the admitted Linux ext4 profile.
    LinuxProfileRequired,
    /// An internal protocol directory name has no component.
    EmptyProtocolName,
    /// The kernel omitted required platform identity observations.
    PlatformStatus {
        /// The required statx mask.
        expected: u32,
        /// The returned statx mask.
        observed: u32,
    },
    /// The filesystem does not satisfy the writable case-sensitive ext4 profile.
    FilesystemProfile {
        /// The required ext4 filesystem type.
        expected_filesystem: i128,
        /// The observed filesystem type.
        observed_filesystem: i128,
        /// The observed mount flags.
        observed_mount_flags: u64,
        /// The observed inode flags.
        observed_inode_flags: u32,
    },
    /// A protocol directory crosses the admitted device or mount.
    MountBoundaryChanged {
        /// The root's device major and minor.
        expected_device: (u32, u32),
        /// The child's device major and minor.
        observed_device: (u32, u32),
        /// The root's mount identity.
        expected_mount: u64,
        /// The child's mount identity.
        observed_mount: u64,
    },
}

impl fmt::Display for FilesystemOperationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "filesystem operation refused: {self:?}")
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
