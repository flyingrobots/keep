//! This module binds retention closure evidence to this store's own catalog head.

use std::io;

use cap_std::fs::Dir;

use super::filesystem_retention_current::read_exact_optional;
use super::{RetentionCurrentStateRefusal, RetentionPublicationPreparation};
use crate::adapters::ChecksummedPublicationHead;

const HEAD_NAME: &str = "HEAD";
const HEAD_LENGTH: usize = crate::adapters::publication_head_decoder::ENCODED_LENGTH;

/// Requires the store's catalog head to name the catalog the closure was verified against.
///
/// A preparation carries a closure verified against one pinned `CatalogSnapshot`,
/// which the caller may have taken from another store. Publication must not
/// proceed unless this store's own `HEAD` names exactly that catalog generation
/// and digest; otherwise the receipt would cite foreign evidence for anchors
/// whose records may be absent from these pools.
pub(super) fn require_current_catalog(
    root: &Dir,
    preparation: &RetentionPublicationPreparation<'_>,
) -> io::Result<()> {
    let closure = preparation.closure();
    let expected_generation = closure.catalog_generation();
    let bytes = read_exact_optional(root, HEAD_NAME, HEAD_LENGTH)?.ok_or_else(|| {
        RetentionCurrentStateRefusal::CatalogDisagreed {
            expected_generation,
            observed_generation: None,
        }
        .into_io()
    })?;
    let head = ChecksummedPublicationHead::decode(&bytes)
        .map_err(|source| RetentionCurrentStateRefusal::CatalogHeadRefused { source }.into_io())?;
    if head.generation() == expected_generation && head.catalog_digest() == closure.catalog_digest()
    {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::CatalogDisagreed {
            expected_generation,
            observed_generation: Some(head.generation()),
        }
        .into_io())
    }
}
