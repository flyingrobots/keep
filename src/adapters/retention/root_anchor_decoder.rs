//! This boundary module owns canonical retention anchor body decoding.

use super::RetentionRootDecodeError;
use crate::{BlobId, LayoutId, RetentionAnchor};

const BLOB_ID_WIDTH: usize = 59;
const ANCHOR_WIDTH: usize = 119;

pub(super) fn decode(
    encoded: &[u8],
    anchor_count: u32,
) -> Result<Vec<RetentionAnchor>, RetentionRootDecodeError> {
    let capacity =
        usize::try_from(anchor_count).map_err(|_| RetentionRootDecodeError::LengthOverflow)?;
    let mut anchors = Vec::new();
    anchors
        .try_reserve_exact(capacity)
        .map_err(|source| RetentionRootDecodeError::Allocation { source })?;
    let mut previous = None;
    for (position, bytes) in encoded.chunks_exact(ANCHOR_WIDTH).enumerate() {
        let index =
            u32::try_from(position).map_err(|_| RetentionRootDecodeError::LengthOverflow)?;
        let (blob_bytes, layout_bytes) = bytes.split_at(BLOB_ID_WIDTH);
        let blob_id = BlobId::parse_binary(blob_bytes)
            .map_err(|source| RetentionRootDecodeError::BlobId { index, source })?;
        let layout_id = LayoutId::parse_binary(layout_bytes)
            .map_err(|source| RetentionRootDecodeError::LayoutId { index, source })?;
        let observed = RetentionAnchor::new(blob_id, layout_id);
        if let Some(prior) = previous
            && observed <= prior
        {
            return Err(RetentionRootDecodeError::NonCanonicalAnchorOrder { index });
        }
        anchors.push(observed);
        previous = Some(observed);
    }
    if anchors.len() == capacity {
        Ok(anchors)
    } else {
        Err(RetentionRootDecodeError::Truncated {
            expected: capacity
                .checked_mul(ANCHOR_WIDTH)
                .ok_or(RetentionRootDecodeError::LengthOverflow)?,
            observed: encoded.len(),
        })
    }
}
