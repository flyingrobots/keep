//! The reference store as a transfer source.

use super::ReferenceStore;
use crate::LayoutId;
use crate::authenticated_read::ChunkReader;
use crate::store::{StreamConsumer, TransferSource, TransferSourceError};

impl crate::store::SealedTransferSource for ReferenceStore {}

impl TransferSource for ReferenceStore {
    fn stream_layout(
        &self,
        layout_id: LayoutId,
        consume: StreamConsumer<'_>,
    ) -> Result<(), TransferSourceError> {
        let layout = self
            .layout(layout_id)
            .ok_or(TransferSourceError::LayoutMissing {
                requested: layout_id,
            })?;
        let mut reader = ChunkReader::new(self, layout_id, layout);
        consume(layout.target(), &mut reader);
        reader.transfer_refusal().map_or(Ok(()), Err)
    }
}
