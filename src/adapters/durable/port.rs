//! The durable adapter's implementations of the content-store port: the
//! snapshot answers the read half; the writer and its staging answer the
//! write half.

use std::io::{Read, Write};

use super::{
    DurableIngestionError, DurableIngestionReceipt, DurableRangeReadReceipt, DurableReadError,
    DurableReconstructionReceipt, DurableSnapshot, DurableStagedBlob, DurableWriter,
};
use crate::{
    BlobId, ByteRange, ContentReads, ContentStaging, LayoutId, StagedContent, StagingLimits,
};

impl ContentReads for DurableSnapshot {
    type ContainsError = super::DurableStoreError;
    type ReconstructionReceipt = DurableReconstructionReceipt;
    type RangeReceipt = DurableRangeReadReceipt;
    type ReconstructionError = DurableReadError;
    type RangeError = DurableReadError;

    fn contains_blob(&self, target: BlobId) -> Result<bool, Self::ContainsError> {
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

impl ContentStaging for DurableWriter {
    type Staged<'store> = DurableStagedBlob<'store>;
    type Error = DurableIngestionError;

    fn stage<'store>(
        &'store mut self,
        source: &mut dyn Read,
        limits: StagingLimits,
    ) -> Result<DurableStagedBlob<'store>, DurableIngestionError> {
        Self::stage(self, source, limits)
    }

    fn stage_expected<'store>(
        &'store mut self,
        source: &mut dyn Read,
        expected: BlobId,
        limits: StagingLimits,
    ) -> Result<DurableStagedBlob<'store>, DurableIngestionError> {
        Self::stage_expected(self, source, expected, limits)
    }
}

impl StagedContent for DurableStagedBlob<'_> {
    type Receipt = DurableIngestionReceipt;
    type Error = DurableIngestionError;

    fn target(&self) -> BlobId {
        Self::target(self)
    }

    fn layout_id(&self) -> LayoutId {
        Self::layout_id(self)
    }

    fn commit(self) -> Result<DurableIngestionReceipt, DurableIngestionError> {
        Self::commit(self)
    }
}
