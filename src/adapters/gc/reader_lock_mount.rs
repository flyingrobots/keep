//! This module owns the reader-lock mount coordinate's distinct type.

/// Opaque mount-instance coordinate observed for `reader.lock`.
///
/// This is same-process evidence; a remount can change it without changing the
/// restart-stable device and file coordinates.
/// Every `u64`, including zero, is representable by the existing wire format.
/// Construction preserves the value without claiming observation or authority.
/// Operations do not allocate, block, or perform I/O.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReaderLockMount(u64);

impl ReaderLockMount {
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
