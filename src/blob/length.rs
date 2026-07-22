//! Logical blob length.

use std::fmt;

/// The exact number of logical bytes named by a blob identity.
///
/// This length describes logical content. It is not an encoded length, segment
/// length, allocation size, or physical range.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobLength(u64);

impl BlobLength {
    /// The length of the empty byte sequence.
    pub const ZERO: Self = Self(0);

    /// Constructs a `BlobLength` from a value a caller has already validated.
    ///
    /// # Preconditions
    ///
    /// This performs no validation of its own. `value` MUST already be
    /// known-lawful for its context — canonical decimal text with no
    /// leading zeroes when decoded from the text codec, or a raw
    /// accumulated byte count from [`BlobHasher`](super::BlobHasher).
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the logical byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns whether the named logical content is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(super) fn checked_add(self, incoming: Self) -> Option<Self> {
        self.0.checked_add(incoming.0).map(Self)
    }
}

impl fmt::Display for BlobLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
