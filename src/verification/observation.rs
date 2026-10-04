//! This module owns bounded coordinates for verification contradictions.

use crate::{BlobId, ProfileBoundary};

/// A required or observed fact at a verification boundary.
///
/// Structural failures retain their precise typed decoder cause in the adapter
/// error; these predicates never replace that cause with a parsed message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationObservation {
    /// Exact chunk content identity.
    Chunk(crate::ChunkId),
    /// Canonical layout identity.
    Layout(crate::LayoutId),
    /// Complete logical byte identity.
    Blob(BlobId),
    /// One expected or replayed profile boundary; absence is meaningful.
    ProfileBoundary(Option<ProfileBoundary>),
    /// Canonical bytes satisfying the boundary's admission contract.
    Canonical,
    /// Bytes refused by the preserved typed decoder or admission error.
    Refused,
}
