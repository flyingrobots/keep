//! Receipt for one authenticated reconstruction.

use crate::{BlobId, BlobLength, LayoutId};

/// Exact identities and length successfully written by reconstruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the reconstruction receipt records authenticated identity and length"]
pub struct ReconstructionReceipt {
    target: BlobId,
    layout_id: LayoutId,
    bytes_written: BlobLength,
}

impl ReconstructionReceipt {
    pub(super) const fn new(
        target: BlobId,
        layout_id: LayoutId,
        bytes_written: BlobLength,
    ) -> Self {
        Self {
            target,
            layout_id,
            bytes_written,
        }
    }

    /// Returns the exact logical identity reconstructed.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// Returns the verified canonical layout identity used.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }

    /// Returns the exact authenticated byte count written.
    #[must_use]
    pub const fn bytes_written(self) -> BlobLength {
        self.bytes_written
    }
}
