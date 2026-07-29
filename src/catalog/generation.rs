//! Checked positive catalog generation.

use std::num::NonZeroU64;

use super::CatalogGenerationError;

/// Positive, canonically ordered catalog-generation coordinate.
///
/// Generation `1` is the first published generation. A successor is admitted
/// only through checked arithmetic; overflow is a typed refusal.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogGeneration(NonZeroU64);

impl CatalogGeneration {
    /// Admits one positive generation.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogGenerationError::Zero`] when `value` is zero.
    pub const fn new(value: u64) -> Result<Self, CatalogGenerationError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(CatalogGenerationError::Zero),
        }
    }

    /// Returns the exact positive generation value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// Derives the next generation through checked addition.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogGenerationError::Exhausted`] when this generation is
    /// `u64::MAX`.
    pub const fn successor(self) -> Result<Self, CatalogGenerationError> {
        let current = self.get();
        let Some(next) = current.checked_add(1) else {
            return Err(CatalogGenerationError::Exhausted { current });
        };
        Self::new(next)
    }
}
