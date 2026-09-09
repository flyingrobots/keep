//! This module owns the bounded retry limit of reader view collection.

use std::num::NonZeroU32;

/// How many times a reader may re-collect before refusing a moving store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct ReaderAttemptLimit(NonZeroU32);

impl ReaderAttemptLimit {
    /// Three attempts: one publication may land between any two reads, and a
    /// store that moves faster than a reader can double-collect is refused.
    pub const DEFAULT: Self = Self(NonZeroU32::MIN.saturating_add(2));

    /// Admits an explicit positive attempt count.
    pub const fn new(attempts: NonZeroU32) -> Self {
        Self(attempts)
    }

    /// The admitted attempt count.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}
