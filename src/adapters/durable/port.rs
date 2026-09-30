//! The durable snapshot's implementation of the content-store read port.

use std::io::Write;

use super::{
    DurableRangeReadReceipt, DurableReadError, DurableReconstructionReceipt, DurableSnapshot,
};
use crate::{BlobId, ByteRange, ContentReads, LayoutId};

impl ContentReads for DurableSnapshot {
    type ReconstructionReceipt = DurableReconstructionReceipt;
    type RangeReceipt = DurableRangeReadReceipt;
    type ReconstructionError = DurableReadError;
    type RangeError = DurableReadError;

    fn contains_blob(&self, target: BlobId) -> bool {
        Self::contains_blob(self, target)
    }

    fn reconstruct(
        &self,
        target: BlobId,
        output: &mut dyn Write,
    ) -> Result<DurableReconstructionReceipt, DurableReadError> {
        Self::reconstruct(self, target, output)
    }

    fn reconstruct_layout(
        &self,
        layout_id: LayoutId,
        output: &mut dyn Write,
    ) -> Result<DurableReconstructionReceipt, DurableReadError> {
        Self::reconstruct_layout(self, layout_id, output)
    }

    fn read_range(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<DurableRangeReadReceipt, DurableReadError> {
        Self::read_range(self, target, requested, output)
    }

    fn read_layout_range(
        &self,
        layout_id: LayoutId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<DurableRangeReadReceipt, DurableReadError> {
        Self::read_layout_range(self, layout_id, requested, output)
    }
}
