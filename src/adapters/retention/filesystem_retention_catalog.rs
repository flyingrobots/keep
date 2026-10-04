//! This module binds retention closure evidence to this store's own catalog head.

use std::io;

use cap_std::fs::Dir;

use cap_fs_ext::DirExt;

use super::filesystem_retention_current::read_exact_optional;
use super::{
    RetentionCurrentStateRefusal, RetentionPublicationPreparation, verify_retention_closure,
};
use crate::adapters::{
    CatalogRestartPolicy, ChecksummedCatalog, ChecksummedPublicationHead, catalog_restart_loader,
    physical_pool_name,
};

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
/// segments are re-read afterwards by [`reverify_closure_members`].
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
    require_selected_catalog(root, head)?;
    Ok(())
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

/// Re-reads every closure member from this store's own pools and re-verifies
/// the candidate's closure against them under this authority.
///
/// The preparation's closure was verified against a `CatalogSnapshot` the
/// caller supplied, which may have been read by another process, from
/// another store, or before a pool entry changed. This loads the catalog
/// `HEAD` selects together with every segment it names, bounded by `policy`,
/// admits each record through the inherited segment laws, and re-runs
/// closure verification. The exact decode or admission error a corrupt member
/// produces travels as the `source` of the refusal; nothing rewraps it into
/// a string.
pub(super) fn reverify_closure_members(
    root: &Dir,
    policy: CatalogRestartPolicy,
    preparation: &RetentionPublicationPreparation<'_>,
) -> io::Result<()> {
    let member_refused = |source| {
        RetentionCurrentStateRefusal::ClosureMemberRefused {
            source: Box::new(source),
        }
        .into_io()
    };
    let loaded = catalog_restart_loader::load_from_directory(root, HEAD_NAME, policy)
        .map_err(member_refused)?;
    let snapshot = loaded.snapshot().map_err(member_refused)?;
    let expected = preparation.closure();
    if (loaded.generation(), loaded.catalog_digest())
        != (expected.catalog_generation(), expected.catalog_digest())
    {
        return Err(RetentionCurrentStateRefusal::CatalogChanged.into_io());
    }
    let observed =
        verify_retention_closure(preparation.candidate().root(), &snapshot).map_err(|source| {
            RetentionCurrentStateRefusal::ClosureReverificationRefused { source }.into_io()
        })?;
    if observed.digest() == expected.digest() {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::ClosureDigestChanged.into_io())
    }
}
