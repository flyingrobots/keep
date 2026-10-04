//! This boundary module owns the semantic verification receipt: the
//! bounded projection of one report or refusal onto one view.

use super::{ReceiptSubject as VerificationSubject, ReceiptVerificationDepth as VerificationDepth};
use crate::adapters::GcRetentionState;
use crate::{BlobId, CatalogDigest, CatalogGeneration, LayoutId};

/// The verification contract version every receipt binds.
pub const VERIFICATION_CONTRACT_VERSION: u32 = 1;

/// The view a verification ran against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationView {
    /// The non-durable reference store.
    Reference,
    /// One admitted durable snapshot.
    Durable {
        /// The catalog generation the view bound.
        catalog_generation: CatalogGeneration,
        /// That catalog's digest.
        catalog_digest: CatalogDigest,
        /// The retention state the view bound.
        retention: GcRetentionState,
    },
}

/// Which evidence a `Missing` refusal named, without its identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptMissing {
    /// No committed layout names the blob.
    Blob,
    /// The exact layout is not committed.
    Layout,
    /// The named layout names a chunk the view lacks.
    Chunk {
        /// The layout naming the chunk.
        layout: LayoutId,
        /// Zero-based entry index.
        index: u64,
    },
}

/// Which contradiction a `Corrupt` refusal named, without the expected and
/// observed identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptCorruption {
    /// A chunk does not hash to the identity the layout names.
    ChunkIdentity {
        /// The layout naming the chunk.
        layout: LayoutId,
        /// Zero-based entry index.
        index: u64,
    },
    /// The committed layout does not produce the identity it is keyed by.
    LayoutIdentity {
        /// The identity the view committed the layout under.
        expected: LayoutId,
    },
    /// The authenticated chunks do not reproduce the target blob identity.
    BlobIdentity {
        /// The layout reconstructed.
        layout: LayoutId,
    },
    /// Replaying the registered storage profile diverged from the layout.
    ProfileBoundary {
        /// The layout reconstructed.
        layout: LayoutId,
        /// Zero-based boundary index at which replay diverged.
        index: u64,
    },
}

/// The refusal a receipt records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptRefusal {
    /// Required evidence is absent from a complete view.
    Missing {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the absence was found.
        stage: VerificationDepth,
        /// What was absent.
        evidence: ReceiptMissing,
    },
    /// Present evidence contradicts the identity it must reproduce.
    Corrupt {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the contradiction was found.
        stage: VerificationDepth,
        /// The contradiction.
        evidence: ReceiptCorruption,
    },
    /// Admitted evidence conflicts.
    Ambiguous {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the conflict was found.
        stage: VerificationDepth,
    },
    /// The view cannot establish the requested depth at all.
    Unsupported {
        /// Subject requested.
        subject: VerificationSubject,
        /// Depth requested.
        requested: VerificationDepth,
        /// Shallowest depth the view establishes.
        supported_minimum: VerificationDepth,
        /// Deepest depth the view establishes.
        supported_maximum: VerificationDepth,
    },
}

/// What the receipt records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationOutcome {
    /// The depth was established against exactly this layout and target.
    Established {
        /// Subject verified.
        subject: VerificationSubject,
        /// Depth established.
        depth: VerificationDepth,
        /// Layout the depth was established through.
        layout: LayoutId,
        /// Target that layout names.
        target: BlobId,
        /// Chunks authenticated.
        chunks_verified: u64,
    },
    /// The requested depth was refused.
    Refused(ReceiptRefusal),
}

/// One replayable verification receipt.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationReceipt {
    view: VerificationView,
    outcome: VerificationOutcome,
}

impl VerificationReceipt {
    /// Reconstructs a receipt from its decoded parts.
    pub(super) const fn from_parts(view: VerificationView, outcome: VerificationOutcome) -> Self {
        Self { view, outcome }
    }

    /// The view the verification ran against.
    #[must_use]
    pub const fn view(&self) -> VerificationView {
        self.view
    }

    /// What the receipt records.
    #[must_use]
    pub const fn outcome(&self) -> VerificationOutcome {
        self.outcome
    }

    /// The subject the receipt names.
    pub const fn subject(&self) -> VerificationSubject {
        match self.outcome {
            VerificationOutcome::Established { subject, .. }
            | VerificationOutcome::Refused(
                ReceiptRefusal::Missing { subject, .. }
                | ReceiptRefusal::Corrupt { subject, .. }
                | ReceiptRefusal::Ambiguous { subject, .. }
                | ReceiptRefusal::Unsupported { subject, .. },
            ) => subject,
        }
    }
}
