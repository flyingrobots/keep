//! This boundary module owns admission of complete interrupted-record history fields.

use super::{RetentionManifestDecodeError, RetentionRootDecodeError};
use crate::{
    LivenessGeneration, RetentionManifest, RetentionManifestDigest, RetentionRoot,
    RetentionRootDigest, RootGeneration,
};

pub(super) fn root(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    let generation = RootGeneration::new(super::root_field_decoder::read_u64(encoded, 32)?)
        .map_err(|source| RetentionRootDecodeError::Generation { source })?;
    let bytes = super::root_field_decoder::read_array(encoded, 116)?;
    let predecessor = (bytes != [0; 32]).then(|| RetentionRootDigest::from_hash(bytes));
    RetentionRoot::admit_predecessor(generation, predecessor)
        .map_err(|source| RetentionRootDecodeError::Semantic { source })
}

pub(super) fn manifest(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    let generation = LivenessGeneration::new(super::manifest_field_decoder::read_u64(encoded, 32)?)
        .map_err(|source| RetentionManifestDecodeError::LivenessGeneration { source })?;
    let bytes = super::manifest_field_decoder::read_array(encoded, 48)?;
    let predecessor = (bytes != [0; 32]).then(|| RetentionManifestDigest::from_hash(bytes));
    RetentionManifest::admit_predecessor(generation, predecessor)
        .map_err(|source| RetentionManifestDecodeError::Semantic { source })
}
