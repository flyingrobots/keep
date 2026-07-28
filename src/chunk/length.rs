//! Exact nonempty chunk length.

use std::fmt;

/// The exact number of bytes named by a chunk identity.
///
/// A `ChunkLength` is always positive. The registered CDC profile further
/// restricts detector output to at most [`FastCdc::MAXIMUM_CHUNK_LENGTH`],
/// but the identity type remains independent of that profile.
///
/// [`FastCdc::MAXIMUM_CHUNK_LENGTH`]: super::FastCdc::MAXIMUM_CHUNK_LENGTH
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkLength(u32);

impl ChunkLength {
    pub(super) const fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    pub(crate) const fn from_wire(value: u32) -> Option<Self> {
        Self::new(value)
    }

    pub(crate) const fn from_validated(value: u32) -> Self {
        Self(value)
    }

    /// Returns the exact chunk byte count.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ChunkLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
