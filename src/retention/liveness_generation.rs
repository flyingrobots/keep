//! This module owns checked global retention liveness generations.

use std::num::NonZeroU64;

use super::LivenessGenerationError;

/// Positive generation of the global retention manifest.
///
/// This coordinate is deliberately distinct from every per-namespace
/// [`RootGeneration`](super::RootGeneration).
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LivenessGeneration(NonZeroU64);

impl LivenessGeneration {
    /// First global retention liveness generation.
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    /// Admits one positive liveness generation.
    ///
    /// # Errors
    ///
    /// Returns [`LivenessGenerationError::Zero`] when `value` is zero.
    pub const fn new(value: u64) -> Result<Self, LivenessGenerationError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(LivenessGenerationError::Zero),
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
    /// Returns [`LivenessGenerationError::Exhausted`] at `u64::MAX`.
    pub const fn successor(self) -> Result<Self, LivenessGenerationError> {
        let current = self.get();
        let Some(next) = current.checked_add(1) else {
            return Err(LivenessGenerationError::Exhausted { current });
        };
        Self::new(next)
    }
}
