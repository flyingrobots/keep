use std::error::Error;
use std::fs;
use std::io;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use crate::{FilesystemEntryKind, FilesystemNamespaceRefusal};

use super::super::filesystem_catalog_stages::FilesystemCatalogStages;
use super::super::test_support::namespace_refusal;
use super::{StoreFixture, admit_writer, restart_policy};

#[test]
fn publisher_refuses_a_non_directory_protocol_namespace() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-nondirectory")?;
    fs::remove_dir(store.staging())?;
    fs::write(store.staging(), [])?;

    let Err(error) = admit_writer(&store) else {
        return Err("writer admission accepted a non-directory namespace".into());
    };

    assert_eq!(
        namespace_refusal(&error),
        Some(FilesystemNamespaceRefusal::WrongKind {
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::File,
        })
    );
    store.remove()
}

#[cfg(unix)]
#[test]
fn publisher_never_follows_a_symbolic_protocol_namespace() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let store = StoreFixture::create("catalog-filesystem-directory-link")?;
    fs::remove_dir(store.staging())?;
    let target = store.path().join("segments");
    symlink(&target, store.staging())?;

    let Err(error) = admit_writer(&store) else {
        return Err("writer admission followed a symbolic namespace".into());
    };

    assert_eq!(
        namespace_refusal(&error),
        Some(FilesystemNamespaceRefusal::WrongKind {
            expected: FilesystemEntryKind::Directory,
            observed: FilesystemEntryKind::Symlink,
        })
    );
    assert!(store.staging().is_symlink());
    store.remove()
}

/// Opens the shared catalog stages directly, past writer admission.
///
/// The stages repeat the no-follow directory check that admission performs,
/// so a namespace replaced after admission still refuses here.
fn open_stages_past_admission(store: &StoreFixture) -> Result<io::Result<()>, Box<dyn Error>> {
    let root = Dir::open_ambient_dir(store.path(), ambient_authority())?;
    Ok(FilesystemCatalogStages::open(&root, restart_policy()?).map(drop))
}

#[test]
fn stages_refuse_a_non_directory_namespace_replaced_after_admission() -> Result<(), Box<dyn Error>>
{
    let store = StoreFixture::create("catalog-stages-nondirectory")?;
    fs::remove_dir(store.staging())?;
    fs::write(store.staging(), [])?;

    let Err(error) = open_stages_past_admission(&store)? else {
        return Err("catalog stages admitted a non-directory namespace".into());
    };

    assert_eq!(error.kind(), io::ErrorKind::NotADirectory);
    store.remove()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn stages_never_follow_a_symbolic_namespace_replaced_after_admission() -> Result<(), Box<dyn Error>>
{
    use std::os::unix::fs::symlink;

    #[cfg(target_os = "linux")]
    const SYMBOLIC_LINK_LOOP_ERROR: i32 = 40;
    #[cfg(target_os = "macos")]
    const SYMBOLIC_LINK_LOOP_ERROR: i32 = 62;

    let store = StoreFixture::create("catalog-stages-directory-link")?;
    fs::remove_dir(store.staging())?;
    symlink(store.path().join("segments"), store.staging())?;

    let Err(error) = open_stages_past_admission(&store)? else {
        return Err("catalog stages followed a symbolic namespace".into());
    };

    assert_eq!(error.raw_os_error(), Some(SYMBOLIC_LINK_LOOP_ERROR));
    store.remove()
}
