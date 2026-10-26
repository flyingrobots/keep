//! This module owns the durable snapshot as a transfer source.
//!
//! The pull reader serves chunks from the pinned catalog and authenticates
//! each one as it is first served; the snapshot's collector fence stays held
//! for as long as the caller holds the snapshot.

use super::snapshot::CatalogChunks;
use super::{DurableReadError, DurableSnapshot};
use crate::LayoutId;
use crate::authenticated_read::ChunkReader;
use crate::store::{StreamConsumer, TransferSource, TransferSourceError};

impl crate::store::SealedTransferSource for DurableSnapshot {}

impl TransferSource for DurableSnapshot {
    fn stream_layout(
        &self,
        layout_id: LayoutId,
        consume: StreamConsumer<'_>,
    ) -> Result<(), TransferSourceError> {
        let catalog = self.catalog().map_err(view_error)?;
        let layout = self.layout(&catalog, layout_id).map_err(view_error)?;
        let chunks = CatalogChunks::new(&catalog);
        let mut reader = ChunkReader::new(&chunks, layout_id, &layout);
        consume(layout.target(), &mut reader);
        reader.transfer_refusal().map_or(Ok(()), Err)
    }
}

/// Keeps a missing layout in the transfer vocabulary and every other view
/// failure as its exact boxed durable cause.
fn view_error(error: DurableReadError) -> TransferSourceError {
    match error {
        DurableReadError::LayoutMissing { requested } => {
            TransferSourceError::LayoutMissing { requested }
        }
        other => TransferSourceError::View(Box::new(other)),
    }
}
