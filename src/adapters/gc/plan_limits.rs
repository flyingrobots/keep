//! This boundary module owns the explicit bounds one GC plan admits.

use std::fmt;
use std::num::NonZeroU32;

/// Explicit ceilings a GC plan must stay within.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcLimits {
    candidates: NonZeroU32,
}

/// A limit outside the protocol's range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcLimitsError {
    /// A zero limit admits no plan.
    Zero,
    /// The limit exceeds the retirement intent's candidate ceiling.
    AboveMaximum {
        /// The requested limit.
        requested: u32,
    },
}

impl fmt::Display for GcLimitsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str("GC candidate limit must be positive"),
            Self::AboveMaximum { requested } => write!(
                formatter,
                "GC candidate limit {requested} exceeds the intent ceiling {}",
                GcLimits::MAXIMUM_CANDIDATES
            ),
        }
    }
}

impl std::error::Error for GcLimitsError {}

impl GcLimits {
    /// The most candidates one retirement intent can name.
    pub const MAXIMUM_CANDIDATES: u32 = 65_536;
    /// The protocol ceiling as a limit.
    pub const MAXIMUM: Self = Self {
        candidates: match NonZeroU32::new(Self::MAXIMUM_CANDIDATES) {
            Some(candidates) => candidates,
            None => NonZeroU32::MIN,
        },
    };

    /// Admits one candidate ceiling.
    ///
    /// # Errors
    ///
    /// Returns [`GcLimitsError`] for zero or a value above the intent ceiling.
    pub const fn new(candidates: u32) -> Result<Self, GcLimitsError> {
        if candidates > Self::MAXIMUM_CANDIDATES {
            return Err(GcLimitsError::AboveMaximum {
                requested: candidates,
            });
        }
        match NonZeroU32::new(candidates) {
            Some(candidates) => Ok(Self { candidates }),
            None => Err(GcLimitsError::Zero),
        }
    }

    /// Returns the candidate ceiling.
    #[must_use]
    pub const fn candidates(self) -> u32 {
        self.candidates.get()
    }
}
