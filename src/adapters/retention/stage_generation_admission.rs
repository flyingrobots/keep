//! This module owns admission of complete generation fields in interrupted stages.

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionRootDecodeError};
use crate::{LivenessGeneration, RootGeneration};

pub(super) fn root(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    if encoded.len() >= 40 {
        let value = super::root_field_decoder::read_u64(encoded, 32)?;
        let _generation = RootGeneration::new(value)
            .map_err(|source| RetentionRootDecodeError::Generation { source })?;
    }
    Ok(())
}

pub(super) fn manifest(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    if encoded.len() >= 40 {
        let value = super::manifest_field_decoder::read_u64(encoded, 32)?;
        let _generation = LivenessGeneration::new(value)
            .map_err(|source| RetentionManifestDecodeError::LivenessGeneration { source })?;
    }
    Ok(())
}

pub(super) fn head(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    if encoded.len() >= 32 {
        let value = super::head_decoder::read_u64(encoded, 24)?;
        let _generation = LivenessGeneration::new(value)
            .map_err(|source| RetentionHeadDecodeError::LivenessGeneration { source })?;
    }
    Ok(())
}
