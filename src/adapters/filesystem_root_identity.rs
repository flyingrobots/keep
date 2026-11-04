//! This module owns one admitted physical filesystem-root coordinate.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FilesystemRootIdentity {
    device: u64,
    mount: u64,
    file: u64,
}

impl FilesystemRootIdentity {
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        any(test, feature = "repository-tasks")
    ))]
    pub(super) const fn new(device: u64, mount: u64, file: u64) -> Self {
        Self {
            device,
            mount,
            file,
        }
    }

    pub(super) const fn device(self) -> u64 {
        self.device
    }

    pub(super) const fn mount(self) -> u64 {
        self.mount
    }

    pub(super) const fn file(self) -> u64 {
        self.file
    }
}

/// Separates Darwin device coordinates from Linux ones in persisted records.
#[cfg(target_os = "macos")]
pub(super) const DARWIN_COORDINATE_DOMAIN: u64 = 0x8000_0000_0000_0000;

/// Returns the part of a device coordinate that survives a remount or reboot.
///
/// Darwin assigns `st_dev` per mount, so reattaching a volume renumbers it and only
/// the coordinate domain remains comparable across a restart.
#[cfg(target_os = "macos")]
pub(super) const fn restart_stable_device(device: u64) -> u64 {
    device & DARWIN_COORDINATE_DOMAIN
}

/// Returns the part of a device coordinate that survives a remount or reboot.
#[cfg(not(target_os = "macos"))]
pub(super) const fn restart_stable_device(device: u64) -> u64 {
    device
}
