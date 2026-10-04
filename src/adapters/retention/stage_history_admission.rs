//! This boundary module owns admission of available interrupted-record history fields.

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionRootDecodeError};
use crate::{
    LivenessGeneration, RetentionManifest, RetentionManifestDigest, RetentionRoot,
    RetentionRootDigest, RootGeneration,
};

pub(super) fn root(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    let generation = RootGeneration::new(super::root_field_decoder::read_u64(encoded, 32)?)
        .map_err(|source| RetentionRootDecodeError::Generation { source })?;
    if encoded.len() < 148 {
        if generation != RootGeneration::INITIAL {
            return Ok(());
        }
        return absent_predecessor(encoded, 116, |offset, expected, observed| {
            RetentionRootDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }
        });
    }
    let bytes = super::root_field_decoder::read_array(encoded, 116)?;
    let predecessor = (bytes != [0; 32]).then(|| RetentionRootDigest::from_hash(bytes));
    RetentionRoot::admit_predecessor(generation, predecessor)
        .map_err(|source| RetentionRootDecodeError::Semantic { source })
}

pub(super) fn manifest(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    let generation = LivenessGeneration::new(super::manifest_field_decoder::read_u64(encoded, 32)?)
        .map_err(|source| RetentionManifestDecodeError::LivenessGeneration { source })?;
    if encoded.len() < 80 {
        if generation != LivenessGeneration::INITIAL {
            return Ok(());
        }
        return absent_predecessor(encoded, 48, |offset, expected, observed| {
            RetentionManifestDecodeError::PrefixByteMismatch {
                offset,
                expected,
                observed,
            }
        });
    }
    let bytes = super::manifest_field_decoder::read_array(encoded, 48)?;
    let predecessor = (bytes != [0; 32]).then(|| RetentionManifestDigest::from_hash(bytes));
    RetentionManifest::admit_predecessor(generation, predecessor)
        .map_err(|source| RetentionManifestDecodeError::Semantic { source })
}

/// Admits an incomplete predecessor after the complete head generation arrived.
pub(super) fn head_prefix(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    let generation = super::head_decoder::read_u64(encoded, 24)?;
    if generation != LivenessGeneration::INITIAL.get() {
        return Ok(());
    }
    absent_predecessor(encoded, 72, |offset, expected, observed| {
        RetentionHeadDecodeError::PrefixByteMismatch {
            offset,
            expected,
            observed,
        }
    })
}

fn absent_predecessor<E>(
    encoded: &[u8],
    start: usize,
    mismatch: impl Fn(usize, u8, u8) -> E,
) -> Result<(), E> {
    super::stage_fixed_field_admission::admit(encoded, &[(start, &[0; 32])], mismatch)
}
