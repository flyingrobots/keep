//! This module owns checked garbage-collection generations.

use std::num::NonZeroU64;

use super::GcGenerationError;

/// Positive generation of one garbage-collection retirement.
///
/// This coordinate is deliberately distinct from every catalog, root, and
/// liveness generation: it counts retirements, not publications.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GcGeneration(NonZeroU64);

impl GcGeneration {
    /// First garbage-collection generation.
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    /// Admits one positive garbage-collection generation.
    ///
    /// # Errors
    ///
    /// Returns [`GcGenerationError::Zero`] when `value` is zero.
    pub const fn new(value: u64) -> Result<Self, GcGenerationError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(GcGenerationError::Zero),
        }
    }

    /// Returns the exact positive generation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// Derives the exact successor through checked addition.
    ///
    /// # Errors
    ///
    /// Returns [`GcGenerationError::Exhausted`] at `u64::MAX`.
    pub const fn successor(self) -> Result<Self, GcGenerationError> {
        let current = self.get();
        let Some(next) = current.checked_add(1) else {
            return Err(GcGenerationError::Exhausted { current });
        };
        Self::new(next)
    }
}
