//! This boundary module owns coordinate admission within an interrupted root anchor.

use super::RetentionRootDecodeError as Error;
use crate::adapters::{blob_id_binary, layout_id_binary};

pub(super) fn admit(bytes: &[u8], index: u32, start: usize) -> Result<(), Error> {
    super::stage_fixed_field_admission::admit(
        bytes,
        &[
            (0, &blob_id_binary::BINARY_MAGIC),
            (16, &blob_id_binary::IDENTITY_VERSION.to_be_bytes()),
            (18, &[blob_id_binary::HASH_ALGORITHM]),
            (59, &layout_id_binary::BINARY_MAGIC),
            (75, &layout_id_binary::IDENTITY_VERSION.to_be_bytes()),
            (77, &layout_id_binary::LAYOUT_CODEC.to_be_bytes()),
        ],
        |offset, expected, observed| {
            start
                .checked_add(offset)
                .map_or(Error::LengthOverflow, |offset| Error::PrefixByteMismatch {
                    offset,
                    expected,
                    observed,
                })
        },
    )?;
    admit_length(bytes, index)
}

fn admit_length(bytes: &[u8], index: u32) -> Result<(), Error> {
    if bytes.len() >= 87 {
        let observed = super::root_field_decoder::read_u64(bytes, 79)?;
        let _length = layout_id_binary::validate_plan_length(observed)
            .map_err(|source| Error::LayoutId { index, source })?;
        return Ok(());
    }
    let Some(available) = bytes.get(79..) else {
        return Ok(());
    };
    let mut completion = [0_u8; 8];
    for (target, observed) in completion.iter_mut().zip(available) {
        *target = *observed;
    }
    let minimum = u64::from_be_bytes(completion);
    let maximum = crate::LayoutRecordLength::MAXIMUM.get();
    if minimum > maximum {
        return Err(Error::LayoutLengthPrefixAboveMaximum {
            index,
            minimum,
            maximum,
        });
    }
    Ok(())
}
