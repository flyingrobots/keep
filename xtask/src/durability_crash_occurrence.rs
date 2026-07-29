//! This module owns repeated crash-point occurrence coordinates.

/// A zero-based occurrence coordinate for a repeated durability transition.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DurabilityCrashOccurrence(u32);

impl DurabilityCrashOccurrence {
    /// The first occurrence of a repeated durability transition.
    pub const FIRST: Self = Self(0);

    /// Creates an occurrence from its zero-based coordinate.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the zero-based coordinate.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
