//! This module owns preserved causes from reference verification operations.

use std::{error::Error, fmt};

use super::ReferenceVerificationContext;
use crate::{
    BlobHashError, ChunkHashError, ChunkingError, LayoutEncodeError, LayoutId, StorageProfileId,
};

/// The original cause and request evidence from a reference verification operation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReferenceVerificationSource {
    /// A precise refusal with its original request and stage.
    Refusal(ReferenceVerificationContext),
    /// Chunk identity calculation could not complete.
    ChunkHash {
        /// The layout naming the chunk.
        layout: LayoutId,
        /// Its zero-based entry index.
        index: usize,
        /// The original hashing error.
        source: ChunkHashError,
    },
    /// Complete logical identity calculation failed.
    BlobHash(BlobHashError),
    /// Canonical layout encoding failed.
    LayoutEncoding(LayoutEncodeError),
    /// The layout has no implemented profile verifier.
    ProfileVerifierUnavailable {
        /// The layout under verification.
        layout: LayoutId,
        /// The unsupported registered profile.
        profile: StorageProfileId,
    },
    /// Profile replay could not complete.
    ProfileChunking {
        /// The layout under verification.
        layout: LayoutId,
        /// The original detector failure.
        source: ChunkingError,
    },
    /// The authenticated entry count exceeded its representable range.
    ChunkCountOverflow,
}

impl fmt::Display for ReferenceVerificationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Refusal(_) => "reference verification refused the supplied evidence",
            Self::ChunkHash { .. } => "reference chunk hashing failed",
            Self::BlobHash(_) => "reference blob hashing failed",
            Self::LayoutEncoding(_) => "reference layout encoding failed",
            Self::ProfileVerifierUnavailable { .. } => "reference profile verifier unavailable",
            Self::ProfileChunking { .. } => "reference profile replay failed",
            Self::ChunkCountOverflow => "reference authenticated chunk count overflowed",
        })
    }
}

impl Error for ReferenceVerificationSource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChunkHash { source, .. } => Some(source),
            Self::BlobHash(source) => Some(source),
            Self::LayoutEncoding(source) => Some(source),
            Self::ProfileChunking { source, .. } => Some(source),
            Self::Refusal(_)
            | Self::ProfileVerifierUnavailable { .. }
            | Self::ChunkCountOverflow => None,
        }
    }
}
