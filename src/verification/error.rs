//! This module owns the two outcomes a verification can return besides a
//! report.

use crate::{
    BlobHashError, ChunkHashError, ChunkingError, LayoutEncodeError, LayoutId, StorageProfileId,
};

use super::VerificationRefusal;

/// Why a verification returned no report.
///
/// A refusal is evidence about content. An operational failure is not: it
/// supports no conclusion about presence, absence, or integrity.
#[derive(Debug)]
pub enum VerificationError {
    /// The view established that the requested depth cannot hold.
    ///
    /// The refusal is boxed because it carries full expected and observed
    /// identities; the success path stays small.
    Refused(Box<VerificationRefusal>),
    /// The operation could not run to a conclusion.
    Operational(VerificationFailure),
}

impl VerificationError {
    /// Wraps one evidenced refusal.
    #[must_use]
    pub fn refused(refusal: VerificationRefusal) -> Self {
        Self::Refused(Box::new(refusal))
    }
}

/// An operational failure during verification.
#[derive(Debug)]
pub enum VerificationFailure {
    /// Stored chunk bytes could not form a lawful chunk identity.
    ChunkHash {
        /// Layout naming the chunk.
        layout: LayoutId,
        /// Zero-based entry index.
        index: usize,
        /// Exact hashing failure.
        source: ChunkHashError,
    },
    /// Logical identity calculation failed.
    BlobHash {
        /// Exact hashing failure.
        source: BlobHashError,
    },
    /// A supplied layout could not produce its canonical record.
    LayoutEncoding {
        /// Exact encoding failure.
        source: LayoutEncodeError,
    },
    /// No verifier is implemented for the layout's registered profile.
    ProfileVerifierUnavailable {
        /// Layout whose profile could not be replayed.
        layout: LayoutId,
        /// Registered profile without a verifier.
        profile: StorageProfileId,
    },
    /// Replaying the registered storage profile failed to run.
    ProfileChunking {
        /// Layout whose profile was replayed.
        layout: LayoutId,
        /// Exact detector failure.
        source: ChunkingError,
    },
}
