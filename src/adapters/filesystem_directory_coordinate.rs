//! This module owns one live filesystem-directory coordinate.

/// Device and mount coordinates of a directory observed by this process.
///
/// Admission compares these only within one operation, to keep every protocol
/// directory on the root's device and mount. They are never persisted, because a
/// store is portable and Darwin renumbers `st_dev` each time a volume mounts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FilesystemDirectoryCoordinate {
    device: u64,
    mount: u64,
}

impl FilesystemDirectoryCoordinate {
    #[cfg(any(target_os = "linux", target_os = "macos", test))]
    pub(super) const fn new(device: u64, mount: u64) -> Self {
        Self { device, mount }
    }

    pub(super) const fn device(self) -> u64 {
        self.device
    }

    pub(super) const fn mount(self) -> u64 {
        self.mount
    }
}
