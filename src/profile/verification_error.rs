//! This module owns storage-profile replay failures independent of adapters.

use crate::{ChunkingError, ProfileBoundary, StorageProfileId};

/// Failure while replaying one admitted storage profile over logical bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling adapters map the same domain replay failures"
)]
pub(crate) enum StorageProfileVerificationError {
    /// No replay verifier implements the admitted profile.
    Unsupported {
        /// Registered profile without a verifier.
        profile: StorageProfileId,
    },
    /// The registered detector refused the supplied byte stream.
    Chunking {
        /// Exact detector failure.
        source: ChunkingError,
    },
    /// Replayed boundaries differ from the admitted layout.
    BoundaryMismatch {
        /// Zero-based boundary index.
        index: usize,
        /// Boundary committed by the layout, or absence for an extra boundary.
        expected: Option<ProfileBoundary>,
        /// Replayed boundary, or absence for a missing boundary.
        observed: Option<ProfileBoundary>,
    },
}
