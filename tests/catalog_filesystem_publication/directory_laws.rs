use std::error::Error;
use std::fs;
use std::io;

use keep::{FilesystemCatalogPublisher, FilesystemWriterLock};

use super::{StoreFixture, restart_policy};

#[test]
fn publisher_refuses_a_non_directory_protocol_namespace() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-nondirectory")?;
    fs::remove_dir(store.staging())?;
    fs::write(store.staging(), [])?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;

    let Err(error) = FilesystemCatalogPublisher::open(lock, restart_policy()?) else {
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

    let Err(error) = FilesystemCatalogPublisher::open(lock, restart_policy()?) else {
        return Err("publisher followed a symbolic namespace".into());
    };

    assert_eq!(error.raw_os_error(), Some(SYMBOLIC_LINK_LOOP_ERROR));
    store.remove()
}
