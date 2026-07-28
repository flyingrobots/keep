//! Receipt for one authenticated exact byte-range read.

use crate::{BlobId, ByteLength, ByteRange, LayoutId};

/// Exact coordinates successfully written from authenticated overlapping chunks.
///
/// This receipt proves that the requested bytes came from chunks whose complete
/// identities were verified under the admitted layout. It does not prove that
/// unrequested chunks, the complete blob identity, or unselected profile
/// boundaries were verified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the range-read receipt records the exact authenticated range"]
pub struct RangeReadReceipt {
    target: BlobId,
    layout_id: LayoutId,
    requested: ByteRange,
    bytes_written: ByteLength,
}

impl RangeReadReceipt {
    pub(super) const fn new(
        target: BlobId,
        layout_id: LayoutId,
        requested: ByteRange,
        bytes_written: ByteLength,
    ) -> Self {
        Self {
            target,
            layout_id,
            requested,
            bytes_written,
        }
    }

    /// Returns the logical blob identity named by the admitted layout.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// Returns the canonical identity of the admitted layout used.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }

    /// Returns the exact requested half-open range.
    #[must_use]
    pub const fn requested(self) -> ByteRange {
        self.requested
    }

    /// Returns the exact authenticated byte count written.
    #[must_use]
    pub const fn bytes_written(self) -> ByteLength {
        self.bytes_written
    }
}
