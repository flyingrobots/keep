//! This module owns one typed logical reconstruction anchor.

use crate::blob::BlobId;
use crate::layout::LayoutId;

/// Exact logical blob and canonical layout coordinates retained together.
///
/// Construction cannot fail because both component identities are already
/// validated. An anchor proves only the requested coordinates; closure
/// admission must separately prove the named layout, chunks, and blob bytes.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionAnchor {
    blob_id: BlobId,
    layout_id: LayoutId,
}

impl RetentionAnchor {
    /// Combines one validated logical blob and layout coordinate.
    pub const fn new(blob_id: BlobId, layout_id: LayoutId) -> Self {
        Self { blob_id, layout_id }
    }

    /// Returns the exact retained logical blob coordinate.
    #[must_use]
    pub const fn blob_id(self) -> BlobId {
        self.blob_id
    }

    /// Returns the exact retained layout coordinate.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }
}
