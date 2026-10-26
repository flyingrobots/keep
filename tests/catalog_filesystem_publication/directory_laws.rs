use std::error::Error;
use std::fs;
use std::io;

use crate::{FilesystemCatalogPublisher, FilesystemWriterLock};

use super::{StoreFixture, restart_policy};

#[test]
fn publisher_refuses_a_non_directory_protocol_namespace() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-nondirectory")?;
    fs::remove_dir(store.staging())?;
    fs::write(store.staging(), [])?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;

    let Err(error) = FilesystemCatalogPublisher::open_unchecked_for_tests(lock, restart_policy()?)
    else {
        return Err("publisher admitted a non-directory namespace".into());
    };

    assert_eq!(error.kind(), io::ErrorKind::NotADirectory);
    store.remove()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn publisher_never_follows_a_symbolic_protocol_namespace() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    #[cfg(target_os = "linux")]
    const SYMBOLIC_LINK_LOOP_ERROR: i32 = 40;
    #[cfg(target_os = "macos")]
    const SYMBOLIC_LINK_LOOP_ERROR: i32 = 62;

    let store = StoreFixture::create("catalog-filesystem-directory-link")?;
    fs::remove_dir(store.staging())?;
    let target = store.path().join("replacement-staging");
    fs::create_dir(&target)?;
    symlink(&target, store.staging())?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;

    let Err(error) = FilesystemCatalogPublisher::open_unchecked_for_tests(lock, restart_policy()?)
    else {
        return Err("publisher followed a symbolic namespace".into());
    };

    assert_eq!(error.raw_os_error(), Some(SYMBOLIC_LINK_LOOP_ERROR));
    store.remove()
}

// Size: medium (owned filesystem). Oracle: a crash-hook prefix longer than the
// encoded artifact is a typed caller error naming the artifact and both lengths.
// Delete when the repository-task prefix hooks are removed.
#[cfg(feature = "repository-tasks")]
#[test]
fn oversized_publication_prefixes_refuse_with_the_artifact_and_bounds() -> Result<(), Box<dyn Error>>
{
    use crate::{
        CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration, CatalogRestartArtifact,
        FilesystemOperationRefusal,
    };

    fn prefix_refusal(result: io::Result<()>) -> Option<(io::ErrorKind, FilesystemOperationRefusal)> {
        let error = result.err()?;
        let refusal = error
            .get_ref()?
            .downcast_ref::<FilesystemOperationRefusal>()
            .copied()?;
        Some((error.kind(), refusal))
    }

    let store = StoreFixture::create("catalog-filesystem-prefix-bound")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher =
        FilesystemCatalogPublisher::open_unchecked_for_tests(lock, restart_policy()?)?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &[])?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let catalog_length = catalog.encoded().len();
    let head_length = head.encoded().len();
    let catalog_over = catalog_length.checked_add(1).ok_or("catalog length overflow")?;
    let head_over = head_length.checked_add(1).ok_or("head length overflow")?;

    assert_eq!(
        prefix_refusal(publisher.write_catalog_prefix_for_repository_tasks(
            &catalog,
            catalog_over
        )),
        Some((
            io::ErrorKind::InvalidInput,
            FilesystemOperationRefusal::PrefixExceedsArtifact {
                artifact: CatalogRestartArtifact::Catalog,
                maximum: catalog_length,
                observed: catalog_over,
            }
        ))
    );
    assert_eq!(
        prefix_refusal(publisher.write_head_prefix_for_repository_tasks(&head, head_over)),
        Some((
            io::ErrorKind::InvalidInput,
            FilesystemOperationRefusal::PrefixExceedsArtifact {
                artifact: CatalogRestartArtifact::Head,
                maximum: head_length,
                observed: head_over,
            }
        ))
    );
    drop(publisher);
    store.remove()
}
