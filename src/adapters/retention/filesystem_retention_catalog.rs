//! This module binds retention closure evidence to this store's own catalog head.

use std::io;

use cap_std::fs::Dir;

use cap_fs_ext::DirExt;

use super::filesystem_retention_current::read_exact_optional;
use super::{RetentionCurrentStateRefusal, RetentionPublicationPreparation};
use crate::adapters::{ChecksummedCatalog, ChecksummedPublicationHead, physical_pool_name};

const HEAD_NAME: &str = "HEAD";
const CATALOGS_NAME: &str = "catalogs";
const HEAD_LENGTH: usize = crate::adapters::publication_head_decoder::ENCODED_LENGTH;

/// Requires the store's catalog head to name the catalog the closure was verified against.
///
/// A preparation carries a closure verified against one pinned `CatalogSnapshot`,
/// which the caller may have taken from another store or before this store
/// lost a pool entry. Publication must not proceed unless this store's own
/// `HEAD` names exactly that catalog generation and digest and the selected
/// catalog pool entry reopens under this authority, bounded by the head's
/// declared length, and decodes to that generation and digest. Closure-member
/// segments are not re-read here: every read authenticates them, and their
/// re-verification under authority belongs to retention recovery.
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
    if head.generation() != expected_generation || head.catalog_digest() != closure.catalog_digest()
    {
        return Err(RetentionCurrentStateRefusal::CatalogDisagreed {
            expected_generation,
            observed_generation: Some(head.generation()),
        }
        .into_io());
    }
    require_selected_catalog(root, head)
}

/// Reopens the catalog pool entry `head` selects and requires it to be that catalog.
fn require_selected_catalog(root: &Dir, head: ChecksummedPublicationHead<'_>) -> io::Result<()> {
    let catalogs = root.open_dir_nofollow(CATALOGS_NAME)?;
    let name = physical_pool_name::catalog(head.generation(), head.catalog_digest());
    let length = usize::try_from(head.catalog_length().get())
        .map_err(|_source| RetentionCurrentStateRefusal::RecordLengthOverflow.into_io())?;
    let bytes = read_exact_optional(&catalogs, &name, length)?
        .ok_or_else(|| RetentionCurrentStateRefusal::CatalogAbsent.into_io())?;
    let catalog = ChecksummedCatalog::decode(&bytes).map_err(|source| {
        RetentionCurrentStateRefusal::CatalogRefused {
            source: Box::new(source),
        }
        .into_io()
    })?;
    if (catalog.generation(), catalog.digest()) == (head.generation(), head.catalog_digest()) {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::CatalogChanged.into_io())
    }
}
