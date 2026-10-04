//! This boundary module owns the physical identity of the exclusively held
//! `reader.lock` that authorized one retirement.

use super::{ReaderLockDevice, ReaderLockFile, ReaderLockMount};

/// Device, mount, and file coordinates of the locked `reader.lock`.
///
/// The mount coordinate is `statx.stx_mnt_id`, a mount instance that changes
/// across unmount, remount, and reboot; like the migration intent's root
/// mount identity it is same-process evidence, and a restart comparison uses
/// the device and file coordinates only.
///
/// Each coordinate has its own type; constructing this value does not prove
/// that the lock was observed or acquired. No allocation, blocking or I/O occurs.
///
/// ```
/// use keep::{ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount};
/// let _identity = ReaderLockIdentity::new(
///     ReaderLockDevice::new(4), ReaderLockMount::new(5), ReaderLockFile::new(6),
/// );
/// ```
///
/// Device and mount cannot be exchanged:
///
/// ```compile_fail,E0308
/// use keep::{ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount};
/// let _identity = ReaderLockIdentity::new(
///     ReaderLockMount::new(5), ReaderLockDevice::new(4), ReaderLockFile::new(6),
/// );
/// ```
///
/// Device and file cannot be exchanged:
///
/// ```compile_fail,E0308
/// use keep::{ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount};
/// let _identity = ReaderLockIdentity::new(
///     ReaderLockFile::new(6), ReaderLockMount::new(5), ReaderLockDevice::new(4),
/// );
/// ```
///
/// Mount and file cannot be exchanged:
///
/// ```compile_fail,E0308
/// use keep::{ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount};
/// let _identity = ReaderLockIdentity::new(
///     ReaderLockDevice::new(4), ReaderLockFile::new(6), ReaderLockMount::new(5),
/// );
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReaderLockIdentity {
    device: ReaderLockDevice,
    mount: ReaderLockMount,
    file: ReaderLockFile,
}

impl ReaderLockIdentity {
    /// Binds the three observed coordinates.
    pub const fn new(
        device: ReaderLockDevice,
        mount: ReaderLockMount,
        file: ReaderLockFile,
    ) -> Self {
        Self {
            device,
            mount,
            file,
        }
    }

    /// Returns the platform device coordinate.
    pub const fn device(self) -> ReaderLockDevice {
        self.device
    }

    /// Returns the platform mount coordinate.
    pub const fn mount(self) -> ReaderLockMount {
        self.mount
    }

    /// Returns the platform file coordinate.
    pub const fn file(self) -> ReaderLockFile {
        self.file
    }
}

/// One `reader.lock` coordinate a receipt failed to bind to its intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderLockCoordinate {
    /// Platform device coordinate.
    Device,
    /// Platform mount coordinate.
    Mount,
    /// Platform file coordinate.
    File,
}
