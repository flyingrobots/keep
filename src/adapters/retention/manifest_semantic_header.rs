//! This boundary module owns post-integrity retention manifest header admission.

use super::RetentionManifestDecodeError;
use super::manifest_header_decoder::DecodedManifestHeader;
use crate::{LivenessGeneration, RetentionManifest, RetentionManifestDigest};

pub(super) struct AdmittedManifestHeader {
    pub(super) generation: LivenessGeneration,
    pub(super) predecessor: Option<RetentionManifestDigest>,
}

pub(super) fn admit(
    header: &DecodedManifestHeader,
) -> Result<AdmittedManifestHeader, RetentionManifestDecodeError> {
    if header.entry_count > RetentionManifest::MAXIMUM_ENTRY_COUNT {
        return Err(RetentionManifestDecodeError::EntryCountExceeded {
            maximum: RetentionManifest::MAXIMUM_ENTRY_COUNT,
            observed: header.entry_count,
        });
    }
    let generation = LivenessGeneration::new(header.generation)
        .map_err(|source| RetentionManifestDecodeError::LivenessGeneration { source })?;
    Ok(AdmittedManifestHeader {
        generation,
        predecessor: predecessor(header.predecessor),
    })
}

fn predecessor(bytes: [u8; 32]) -> Option<RetentionManifestDigest> {
    if bytes == [0_u8; 32] {
        None
    } else {
        Some(RetentionManifestDigest::from_hash(bytes))
    }
}
