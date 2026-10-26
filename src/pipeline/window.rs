//! This boundary module owns the transfer window: how many segments the
//! sink may hold before it must acknowledge them.

use std::num::NonZeroU32;

/// The number of segments applied between sink acknowledgements.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferWindow(NonZeroU32);

impl TransferWindow {
    /// Acknowledge after every segment.
    pub const ONE: Self = Self(NonZeroU32::MIN);

    /// A window of `segments`, refused at zero.
    #[must_use]
    pub const fn new(segments: u32) -> Option<Self> {
        match NonZeroU32::new(segments) {
            Some(segments) => Some(Self(segments)),
            None => None,
        }
    }

    /// The segment count.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}
