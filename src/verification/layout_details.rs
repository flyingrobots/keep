//! This module owns admitted layout-verification work recorded beside a claim.

use crate::{BlobId, LayoutId};

/// Layout coordinates and actual chunk work from a successful reference verification.
///
/// These details cannot be manufactured by callers. They supplement the report's
/// exact subject and depth; they do not grant durable catalog or retention provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceVerificationDetails {
    layout: LayoutId,
    target: BlobId,
    chunks_verified: u64,
}

impl ReferenceVerificationDetails {
    /// The selected committed layout, or the canonical caller-supplied layout.
    pub const fn layout(self) -> LayoutId {
        self.layout
    }

    /// The target named by the verified layout.
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// The number of layout entries whose chunk bytes were authenticated.
    pub const fn chunks_verified(self) -> u64 {
        self.chunks_verified
    }

    pub(super) const fn new(layout: LayoutId, target: BlobId, chunks_verified: u64) -> Self {
        Self {
            layout,
            target,
            chunks_verified,
        }
    }
}
