//! This boundary module owns ordering feasibility of an incomplete canonical anchor.

use super::RetentionRootDecodeError as Error;
use crate::{LayoutRecordLength, RetentionAnchor};

pub(super) fn admit(
    bytes: &[u8],
    index: u32,
    previous: Option<RetentionAnchor>,
) -> Result<(), Error> {
    let Some(prior) = previous else {
        return Ok(());
    };
    let mut greatest = greatest_coordinate(prior)?;
    for (target, observed) in greatest.iter_mut().zip(bytes) {
        *target = *observed;
    }
    let upper = super::root_field_decoder::read_u64(&greatest, 79)?;
    let length = LayoutRecordLength::greatest_at_most(upper)
        .ok_or(Error::NonCanonicalAnchorOrder { index })?;
    greatest
        .get_mut(79..87)
        .ok_or(Error::LengthOverflow)?
        .copy_from_slice(&length.get().to_be_bytes());
    // Clipping and alignment must never replace an available byte. The bound
    // is only a witness for possibility, never returned as an observed anchor.
    if !greatest.starts_with(bytes) {
        return Err(Error::NonCanonicalAnchorOrder { index });
    }
    super::root_anchor_decoder::admit_anchor(&greatest, index, previous).map(|_| ())
}

fn greatest_coordinate(prior: RetentionAnchor) -> Result<[u8; 119], Error> {
    let mut encoded = [u8::MAX; 119];
    encoded
        .get_mut(..59)
        .ok_or(Error::LengthOverflow)?
        .copy_from_slice(&prior.blob_id().encode_binary());
    encoded
        .get_mut(59..)
        .ok_or(Error::LengthOverflow)?
        .copy_from_slice(&prior.layout_id().encode_binary());
    encoded
        .get_mut(19..59)
        .ok_or(Error::LengthOverflow)?
        .fill(u8::MAX);
    encoded
        .get_mut(79..119)
        .ok_or(Error::LengthOverflow)?
        .fill(u8::MAX);
    Ok(encoded)
}
