//! This module owns physical store-root recovery coordinates.

use crate::adapters::filesystem_root_identity::FilesystemRootIdentity;

macro_rules! root_identity {
    ($name:ident, $documentation:literal) => {
        #[doc = $documentation]
        #[must_use]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            /// Returns the exact serialized platform coordinate.
            ///
            /// This value remains a comparison coordinate until a platform
            /// adapter revalidates it against the opened store root.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }

            pub(super) const fn from_admitted(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

root_identity!(
    StoreRootDeviceIdentity,
    "Platform device identity bound into a migration intent."
);
root_identity!(
    StoreRootMountIdentity,
    "Platform mount identity bound into a migration intent."
);
root_identity!(
    StoreRootFileIdentity,
    "Platform file identity bound into a migration intent."
);

#[derive(Clone, Copy)]
pub(super) struct StoreRootIdentities {
    device: StoreRootDeviceIdentity,
    mount: StoreRootMountIdentity,
    file: StoreRootFileIdentity,
}

impl StoreRootIdentities {
    pub(super) const fn new(
        device: StoreRootDeviceIdentity,
        mount: StoreRootMountIdentity,
        file: StoreRootFileIdentity,
    ) -> Self {
        Self {
            device,
            mount,
            file,
        }
    }

    pub(super) const fn from_filesystem(identity: FilesystemRootIdentity) -> Self {
        Self::new(
            StoreRootDeviceIdentity::from_admitted(identity.device()),
            StoreRootMountIdentity::from_admitted(identity.mount()),
            StoreRootFileIdentity::from_admitted(identity.file()),
        )
    }

    pub(super) const fn device(self) -> StoreRootDeviceIdentity {
        self.device
    }

    pub(super) const fn mount(self) -> StoreRootMountIdentity {
        self.mount
    }

    pub(super) const fn file(self) -> StoreRootFileIdentity {
        self.file
    }
}
