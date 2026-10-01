//! This boundary module owns the physical identity of the exclusively held
//! `reader.lock` that authorized one retirement.

/// Device, mount, and file coordinates of the locked `reader.lock`.
///
/// The mount coordinate is `statx.stx_mnt_id`, a mount instance that changes
/// across unmount, remount, and reboot; like the migration intent's root
/// mount identity it is same-process evidence, and a restart comparison uses
/// the device and file coordinates only.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReaderLockIdentity {
    device: u64,
    mount: u64,
    file: u64,
}

impl ReaderLockIdentity {
    /// Binds the three observed coordinates.
    pub const fn new(device: u64, mount: u64, file: u64) -> Self {
        Self {
            device,
            mount,
            file,
        }
    }

    /// Returns the platform device coordinate.
    #[must_use]
    pub const fn device(self) -> u64 {
        self.device
    }

    /// Returns the platform mount coordinate.
    #[must_use]
    pub const fn mount(self) -> u64 {
        self.mount
    }

    /// Returns the platform file coordinate.
    #[must_use]
    pub const fn file(self) -> u64 {
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
