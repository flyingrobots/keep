//! This module owns admission of available retention stage fields.

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionRootDecodeError};

pub(super) fn root(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    admit(
        encoded,
        &[
            (0, b"KEEP:RET:ROOT2\0\0"),
            (16, &2_u16.to_be_bytes()),
            (18, &192_u16.to_be_bytes()),
            (20, &[0; 4]),
            (42, &119_u16.to_be_bytes()),
            (98, &[0; 2]),
            (180, &[0; 12]),
        ],
        |offset, expected, observed| RetentionRootDecodeError::PrefixByteMismatch {
            offset,
            expected,
            observed,
        },
    )?;
    super::stage_generation_admission::root(encoded)?;
    if encoded.len() >= 42 {
        let length = usize::from(super::root_field_decoder::read_u16(encoded, 40)?);
        let _length = crate::RetentionNamespace::admit_length(length)
            .map_err(|source| RetentionRootDecodeError::Namespace { source })?;
    }
    if encoded.len() >= 48 {
        super::root_header_decoder::admit_prefix_length(encoded)?;
    }
    Ok(())
}

pub(super) fn manifest(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    admit(
        encoded,
        &[
            (0, b"KEEP:RET:LIVE2\0\0"),
            (16, &2_u16.to_be_bytes()),
            (18, &160_u16.to_be_bytes()),
            (20, &[0; 4]),
            (40, &72_u16.to_be_bytes()),
            (42, &[0; 2]),
            (112, &[0; 48]),
        ],
        |offset, expected, observed| RetentionManifestDecodeError::PrefixByteMismatch {
            offset,
            expected,
            observed,
        },
    )?;
    super::stage_generation_admission::manifest(encoded)?;
    if encoded.len() >= 48 {
        super::manifest_header_decoder::admit_prefix_length(encoded)?;
    }
    Ok(())
}

pub(super) fn head(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    admit(
        encoded,
        &[
            (0, &super::head_decoder::MAGIC),
            (16, &super::head_decoder::VERSION.to_be_bytes()),
            (18, &super::head_decoder::RECORD_LENGTH.to_be_bytes()),
            (20, &[0; 4]),
            (104, &[0; 8]),
        ],
        |offset, expected, observed| RetentionHeadDecodeError::PrefixByteMismatch {
            offset,
            expected,
            observed,
        },
    )?;
    super::stage_generation_admission::head(encoded)?;
    if encoded.len() >= 40 {
        let value = super::head_decoder::read_u64(encoded, 32)?;
        let _length = crate::RetentionManifestLength::new(value)
            .map_err(|source| RetentionHeadDecodeError::ManifestLength { source })?;
    }
    if encoded.len() >= 104 {
        let _head = super::head_decoder::admit_fields(encoded)?;
    }
    if let Some(preimage) = encoded.get(..super::head_decoder::CHECKSUM_OFFSET) {
        let checksum = super::head_decoder::checksum(preimage);
        admit(
            encoded,
            &[(super::head_decoder::CHECKSUM_OFFSET, &checksum)],
            |offset, expected, observed| RetentionHeadDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            },
        )?;
    }
    Ok(())
}

fn admit<E>(
    encoded: &[u8],
    fields: &[(usize, &[u8])],
    mismatch: impl Fn(usize, u8, u8) -> E,
) -> Result<(), E> {
    for &(start, canonical) in fields {
        for (&expected, (offset, &observed)) in
            canonical.iter().zip(encoded.iter().enumerate().skip(start))
        {
            if expected != observed {
                return Err(mismatch(offset, expected, observed));
            }
        }
    }
    Ok(())
}
