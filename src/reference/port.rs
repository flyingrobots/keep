//! The reference store's implementation of the content-store port: both
//! halves, non-durable by construction.

use std::io::{Read, Write};

use super::{
    IngestionError, PublishError, PublishedBlob, RangeReadError, RangeReadReceipt,
    ReconstructionError, ReconstructionReceipt, ReferenceStore, StagedBlob,
};
use crate::{
    BlobId, ByteRange, CommitReceipt, ContentReads, ContentStaging, LayoutId, StagedContent,
    StagingLimits,
};

impl ContentReads for ReferenceStore {
    type ReconstructionReceipt = ReconstructionReceipt;
    type RangeReceipt = RangeReadReceipt;
    type ReconstructionError = ReconstructionError;
    type RangeError = RangeReadError;

    fn contains_blob(&self, target: BlobId) -> bool {
        Self::contains_blob(self, target)
    }

    fn reconstruct(
        &self,
        target: BlobId,
        output: &mut dyn Write,
    ) -> Result<ReconstructionReceipt, ReconstructionError> {
        Self::reconstruct(self, target, output)
    }

    fn reconstruct_layout(
        &self,
        layout_id: LayoutId,
        output: &mut dyn Write,
    ) -> Result<ReconstructionReceipt, ReconstructionError> {
        Self::reconstruct_layout(self, layout_id, output)
    }

    fn read_range(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<RangeReadReceipt, RangeReadError> {
        Self::read_range(self, target, requested, output)
    }

    fn read_layout_range(
        &self,
        layout_id: LayoutId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<RangeReadReceipt, RangeReadError> {
        Self::read_layout_range(self, layout_id, requested, output)
    }
}

impl ContentStaging for ReferenceStore {
    type Staged = StagedBlob;
    type Error = IngestionError;

    fn stage(
        &self,
        source: &mut dyn Read,
        limits: StagingLimits,
    ) -> Result<StagedBlob, IngestionError> {
        self.stage_bounded(source, limits)
    }

    fn stage_expected(
        &self,
        source: &mut dyn Read,
        expected: BlobId,
        limits: StagingLimits,
    ) -> Result<StagedBlob, IngestionError> {
        let staged = self.stage_bounded(source, limits)?;
        let observed = staged.target();
        if observed != expected {
            return Err(IngestionError::BlobIdentityMismatch { expected, observed });
        }
        Ok(staged)
    }
}

impl StagedContent for StagedBlob {
    type Store = ReferenceStore;
    type Receipt = PublishedBlob;
    type Error = PublishError;

    fn target(&self) -> BlobId {
        Self::target(self)
    }

    fn layout_id(&self) -> LayoutId {
        Self::layout_id(self)
    }

    fn commit(self, store: &mut ReferenceStore) -> Result<PublishedBlob, PublishError> {
        Self::commit(self, store)
    }
}

impl CommitReceipt for PublishedBlob {
    fn target(&self) -> BlobId {
        Self::target(*self)
    }

    fn layout_id(&self) -> LayoutId {
        Self::layout_id(*self)
    }
}
