//! Closed registered storage-profile set.

use super::{StorageProfileAdmissionError, StorageProfileId};
use crate::{ChunkLength, FastCdc};

const FAST_CDC_64K_V1_DIGEST: [u8; 32] = [
    0xaa, 0xfa, 0x6f, 0x05, 0xbd, 0xc8, 0x89, 0x43, 0x06, 0xab, 0xd4, 0x1e, 0xc6, 0xf2, 0xb3, 0xb7,
    0x6c, 0xde, 0x99, 0x5f, 0x25, 0x98, 0xfa, 0x3f, 0xd5, 0x47, 0xd8, 0x1f, 0xbe, 0x1a, 0x34, 0xeb,
];

/// One deterministic storage profile implemented by this Keep version.
///
/// The type has private representation so admitting more profiles remains an
/// additive registry change rather than an exhaustive-enum compatibility
/// break.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegisteredStorageProfile {
    id: StorageProfileId,
    minimum: ChunkLength,
    target: ChunkLength,
    maximum: ChunkLength,
}

impl RegisteredStorageProfile {
    /// The frozen `fastcdc-64k-v1` profile from ADR-0003.
    pub const FAST_CDC_64K_V1: Self = Self {
        id: StorageProfileId::from_validated_digest(FAST_CDC_64K_V1_DIGEST),
        minimum: FastCdc::MINIMUM_CHUNK_LENGTH,
        target: FastCdc::TARGET_CHUNK_LENGTH,
        maximum: FastCdc::MAXIMUM_CHUNK_LENGTH,
    };

    /// Admits a canonical identity when this Keep version implements it.
    ///
    /// # Errors
    ///
    /// Returns [`StorageProfileAdmissionError::Unsupported`] for every
    /// canonical but unregistered identity.
    pub fn admit(id: StorageProfileId) -> Result<Self, StorageProfileAdmissionError> {
        if id == Self::FAST_CDC_64K_V1.id {
            return Ok(Self::FAST_CDC_64K_V1);
        }
        Err(StorageProfileAdmissionError::Unsupported { observed: id })
    }

    /// Returns the immutable canonical profile identity.
    #[must_use]
    pub const fn id(self) -> StorageProfileId {
        self.id
    }

    /// Returns the minimum lawful nonfinal chunk length.
    #[must_use]
    pub const fn minimum_chunk_length(self) -> ChunkLength {
        self.minimum
    }

    /// Returns the target boundary transition length.
    #[must_use]
    pub const fn target_chunk_length(self) -> ChunkLength {
        self.target
    }

    /// Returns the hard maximum chunk length.
    #[must_use]
    pub const fn maximum_chunk_length(self) -> ChunkLength {
        self.maximum
    }
}
