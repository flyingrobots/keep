//! This module owns physical store-root recovery coordinates.

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
