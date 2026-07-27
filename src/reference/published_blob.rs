//! Receipt for one visible reference-store publication.

use crate::{BlobId, LayoutId};

/// Semantic identities made visible by an explicit reference-store commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishedBlob {
    target: BlobId,
    layout_id: LayoutId,
}

impl PublishedBlob {
    pub(super) const fn new(target: BlobId, layout_id: LayoutId) -> Self {
        Self { target, layout_id }
    }

    /// Returns the exact logical blob identity.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// Returns the exact canonical layout identity.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }
}
