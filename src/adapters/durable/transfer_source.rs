//! The durable snapshot as a transfer source.

use super::snapshot::CatalogChunks;
use super::{DurableReadError, DurableSnapshot};
use crate::LayoutId;
use crate::adapters::{StreamConsumer, TransferSource, TransferSourceError};
use crate::authenticated_read::ChunkReader;

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

fn view_error(error: DurableReadError) -> TransferSourceError {
    match error {
        DurableReadError::LayoutMissing { requested } => {
            TransferSourceError::LayoutMissing { requested }
        }
        other => TransferSourceError::View(Box::new(other)),
    }
}
