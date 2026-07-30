//! This module owns checked retention root-generation coordinates.

use std::num::NonZeroU64;

use super::RootGenerationError;

/// Positive generation of one retention namespace root.
///
/// Generation `1` is initial. Empty retained sets still publish a successor;
/// generations are never reused.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootGeneration(NonZeroU64);

impl RootGeneration {
    /// Admits one positive root generation.
    ///
    /// # Errors
    ///
    /// Returns [`RootGenerationError::Zero`] when `value` is zero.
    pub const fn new(value: u64) -> Result<Self, RootGenerationError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(RootGenerationError::Zero),
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
    /// Returns [`RootGenerationError::Exhausted`] at `u64::MAX`.
    pub const fn successor(self) -> Result<Self, RootGenerationError> {
        let current = self.get();
        let Some(next) = current.checked_add(1) else {
            return Err(RootGenerationError::Exhausted { current });
        };
        Self::new(next)
    }
}
