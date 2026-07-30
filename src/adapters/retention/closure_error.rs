//! This module owns typed failures from retention-closure verification.

use std::error::Error;
use std::fmt;

use crate::{
    BlobHashError, BlobId, ChunkingError, LayoutDecodeError, LayoutEntryLimitError, LayoutId,
    ProfileBoundary, RetentionClosureCounter, SegmentRecordIdentity, StorageProfileId,
};

/// Failure to derive and authenticate one complete retained root closure.
#[derive(Debug)]
pub enum RetentionClosureVerificationError {
    /// A checked resource counter overflowed before the next operation.
    CounterOverflow {
        /// Counter whose addition failed.
        counter: RetentionClosureCounter,
        /// Value before the failed addition.
        current: u64,
        /// Requested increment.
        incoming: u64,
    },
    /// A candidate resource observation exceeds the stored admitted limit.
    LimitExceeded {
        /// Counter whose limit was exceeded.
        counter: RetentionClosureCounter,
        /// Stored admitted maximum.
        maximum: u64,
        /// Candidate observed value.
        observed: u64,
    },
    /// The admitted node limit could not become a host-independent entry cap.
    LayoutEntryLimitHostWidth {
        /// Admitted node limit that did not fit the layout cap width.
        observed: u64,
    },
    /// The derived layout entry cap violated the layout protocol bound.
    LayoutEntryLimit {
        /// Exact layout-bound refusal.
        source: LayoutEntryLimitError,
    },
    /// The pinned catalog omits a first-scheduled closure member.
    MissingMember {
        /// Exact missing logical record identity.
        identity: SegmentRecordIdentity,
    },
    /// A selected layout failed bounded canonical decoding.
    LayoutDecode {
        /// Layout named by the retained anchor.
        layout: LayoutId,
        /// Exact decoding refusal.
        source: LayoutDecodeError,
    },
    /// A selected layout names another logical blob.
    AnchorTargetMismatch {
        /// Layout named by the retained anchor.
        layout: LayoutId,
        /// Blob named by the anchor.
        expected: BlobId,
        /// Blob embedded in the admitted layout.
        observed: BlobId,
    },
    /// No replay verifier implements the layout's registered storage profile.
    ProfileVerifierUnavailable {
        /// Layout whose profile could not be replayed.
        layout: LayoutId,
        /// Registered profile without a verifier.
        profile: StorageProfileId,
    },
    /// Replaying the registered storage profile failed.
    ProfileChunking {
        /// Layout whose profile was replayed.
        layout: LayoutId,
        /// Exact detector failure.
        source: ChunkingError,
    },
    /// Replayed profile boundaries differ from the admitted layout.
    ProfileBoundaryMismatch {
        /// Layout whose profile was replayed.
        layout: LayoutId,
        /// Zero-based boundary index.
        index: usize,
        /// Boundary committed by the layout, or absence for an extra boundary.
        expected: Option<ProfileBoundary>,
        /// Replayed boundary, or absence for a missing boundary.
        observed: Option<ProfileBoundary>,
    },
    /// Complete logical identity calculation failed.
    BlobHash {
        /// Layout whose bytes were hashed.
        layout: LayoutId,
        /// Exact hashing failure.
        source: BlobHashError,
    },
    /// Reconstructed bytes do not authenticate as the retained blob.
    BlobIdentityMismatch {
        /// Layout whose complete stream was verified.
        layout: LayoutId,
        /// Blob named by the retained anchor.
        expected: BlobId,
        /// Blob calculated from the selected chunks.
        observed: BlobId,
    },
}

impl Error for RetentionClosureVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutEntryLimit { source } => Some(source),
            Self::LayoutDecode { source, .. } => Some(source),
            Self::ProfileChunking { source, .. } => Some(source),
            Self::BlobHash { source, .. } => Some(source),
            Self::CounterOverflow { .. }
            | Self::LimitExceeded { .. }
            | Self::LayoutEntryLimitHostWidth { .. }
            | Self::MissingMember { .. }
            | Self::AnchorTargetMismatch { .. }
            | Self::ProfileVerifierUnavailable { .. }
            | Self::ProfileBoundaryMismatch { .. }
            | Self::BlobIdentityMismatch { .. } => None,
        }
    }
}

impl fmt::Display for RetentionClosureVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::closure_error_display::display(self, formatter)
    }
}
