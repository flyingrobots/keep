//! This module owns the reader-lock file coordinate's distinct type.

/// Opaque file coordinate of the filesystem object used as `reader.lock`.
///
/// It is a restart-stable coordinate only when observed and compared under the
/// storage protocol; the value alone is not proof of a live filesystem object.
/// Every `u64`, including zero, is representable by the existing wire format.
/// Construction preserves the value without claiming observation or authority.
/// Operations do not allocate, block, or perform I/O.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReaderLockFile(u64);

impl ReaderLockFile {
    /// Records one exact coordinate without additional validity claims.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact coordinate for comparison or canonical encoding.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
