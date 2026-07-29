use std::error::Error;
use std::fs;
use std::io;

use crate::{FilesystemCatalogPublisher, FilesystemWriterLock};

use super::{StoreFixture, restart_policy};

#[test]
fn publisher_has_no_unadmitted_production_constructor() -> Result<(), Box<dyn Error>> {
    let publisher = include_str!("../../src/adapters/filesystem_catalog_publisher.rs");
    let admission = include_str!("../../src/adapters/filesystem_platform_admission.rs");
    if !publisher.contains("pub fn open(\n        admission: FilesystemPlatformAdmission,") {
        return Err("publisher construction does not require platform admission".into());
    }
    let public_items = admission
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("pub "))
        .collect::<Vec<_>>();
    if public_items != ["pub struct FilesystemPlatformAdmission {"] {
        return Err("platform admission exposes an unverified public producer".into());
    }
    Ok(())
}

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
