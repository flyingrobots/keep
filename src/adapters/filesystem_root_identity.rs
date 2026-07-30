//! This module owns one admitted physical filesystem-root coordinate.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FilesystemRootIdentity {
    device: u64,
    mount: u64,
    file: u64,
}

impl FilesystemRootIdentity {
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
