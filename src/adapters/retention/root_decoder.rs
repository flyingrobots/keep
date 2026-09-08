//! This boundary module owns canonical retention root decoding order.

use super::root_header_decoder::HEADER_LENGTH;
use super::{
    AdmittedRetentionRoot, RetentionRootDecodeError, root_anchor_decoder, root_header_decoder,
    root_integrity, root_semantic_header,
};
use crate::{RetentionNamespace, RetentionPolicy, RetentionRoot, RetentionRootDigest};

pub(super) fn decode(
    encoded: &[u8],
) -> Result<AdmittedRetentionRoot<'_>, RetentionRootDecodeError> {
    let header = root_header_decoder::decode(encoded)?;
    let digest = root_integrity::verify(encoded, header.digest_offset, header.checksum_offset)?;
    let namespace_end = HEADER_LENGTH
        .checked_add(header.namespace_length)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    let namespace_bytes =
        encoded
            .get(HEADER_LENGTH..namespace_end)
            .ok_or(RetentionRootDecodeError::Truncated {
                expected: namespace_end,
                observed: encoded.len(),
            })?;
    let anchor_bytes = encoded.get(namespace_end..header.digest_offset).ok_or(
        RetentionRootDecodeError::Truncated {
            expected: header.digest_offset,
            observed: encoded.len(),
        },
    )?;
    let anchor_set_digest = root_integrity::verify_anchor_set(
        header.anchor_count,
        anchor_bytes,
        header.anchor_set_digest,
    )?;
    let admitted_header = root_semantic_header::admit(&header)?;
    let namespace = RetentionNamespace::try_from(namespace_bytes)
        .map_err(|source| RetentionRootDecodeError::Namespace { source })?;
    let anchors = root_anchor_decoder::decode(anchor_bytes, header.anchor_count)?;
    let policy = RetentionPolicy::new(admitted_header.profile, admitted_header.limits);
    let root = RetentionRoot::new(
        namespace,
        admitted_header.generation,
        policy,
        admitted_header.predecessor,
        anchors,
    )
    .map_err(|source| RetentionRootDecodeError::Semantic { source })?;
    Ok(AdmittedRetentionRoot::admitted(
        encoded,
        root,
        anchor_set_digest,
        RetentionRootDigest::from_hash(digest),
    ))
}
