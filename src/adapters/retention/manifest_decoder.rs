//! This boundary module owns canonical retention manifest decoding order.

use super::manifest_header_decoder::HEADER_LENGTH;
use super::{
    AdmittedRetentionManifest, RetentionManifestDecodeError, manifest_entry_decoder,
    manifest_header_decoder, manifest_integrity, manifest_semantic_header,
};
use crate::{RetentionManifest, RetentionManifestDigest};

pub(super) fn decode(
    encoded: &[u8],
) -> Result<AdmittedRetentionManifest<'_>, RetentionManifestDecodeError> {
    let header = manifest_header_decoder::decode(encoded)?;
    let digest = manifest_integrity::verify(encoded, header.digest_offset, header.checksum_offset)?;
    let entry_bytes = encoded.get(HEADER_LENGTH..header.digest_offset).ok_or(
        RetentionManifestDecodeError::Truncated {
            expected: header.digest_offset,
            observed: encoded.len(),
        },
    )?;
    manifest_integrity::verify_entry_set(header.entry_count, entry_bytes, header.entry_set_digest)?;
    let admitted_header = manifest_semantic_header::admit(&header)?;
    let entries = manifest_entry_decoder::decode(entry_bytes, header.entry_count)?;
    let manifest = RetentionManifest::new(
        admitted_header.generation,
        admitted_header.predecessor,
        entries,
    )
    .map_err(|source| RetentionManifestDecodeError::Semantic { source })?;
    Ok(AdmittedRetentionManifest::admitted(
        encoded,
        manifest,
        RetentionManifestDigest::from_hash(digest),
    ))
}
