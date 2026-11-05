//! An initialized store survives its volume being detached and reattached.
//!
//! Size: large. Oracle: Darwin assigns `st_dev` per mount and reuses a freed number,
//! so the law mounts a second volume in the gap to force a new number, while the
//! root inode persists. It needs runner-owned disk images: one holding the scratch
//! root, named by `KEEP_TEST_REATTACH_IMAGE` and `KEEP_TEST_REATTACH_MOUNT`, and one
//! named by `KEEP_TEST_REATTACH_OCCUPIER` and `KEEP_TEST_REATTACH_OCCUPIER_MOUNT`.
//! It is inert otherwise. Portability means no record of the old number can matter;
//! this law keeps that true on a real renumbered volume.

#![cfg(all(feature = "repository-tasks", target_os = "macos"))]

#[allow(dead_code, reason = "this law uses only the initialized-store fixture")]
#[path = "reader_fence_process/fixture.rs"]
pub mod fixture;
#[allow(
    dead_code,
    reason = "this law uses only the fixture's subset of the builder"
)]
#[path = "support/populated_store.rs"]
pub mod populated_store;
#[allow(dead_code, reason = "this law uses Drop cleanup")]
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;

use std::{
    env, error::Error, ffi::OsStr, fs, os::unix::fs::MetadataExt, path::Path, process::Command,
};

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot, ReaderAttemptLimit,
    SegmentReadPolicy,
};

fn hdiutil(arguments: &[&OsStr]) -> Result<(), Box<dyn Error>> {
    if Command::new("/usr/bin/hdiutil")
        .args(arguments)
        .status()?
        .success()
    {
        return Ok(());
    }
    Err("hdiutil failed".into())
}

fn attach(image: &OsStr, mount: &OsStr) -> Result<(), Box<dyn Error>> {
    hdiutil(&[
        "attach".as_ref(),
        "-nobrowse".as_ref(),
        "-mountpoint".as_ref(),
        mount,
        image,
    ])
}

fn load(root: &Path) -> Result<(), Box<dyn Error>> {
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    drop(FilesystemRetentionSnapshot::load(
        root,
        policy,
        ReaderAttemptLimit::DEFAULT,
        keep::StoreExpectation::NoExpectation,
    )?);
    Ok(())
}

#[test]
fn a_reattached_volume_reopens_its_store() -> Result<(), Box<dyn Error>> {
    let (Some(image), Some(mount), Some(occupier), Some(occupier_mount)) = (
        env::var_os("KEEP_TEST_REATTACH_IMAGE"),
        env::var_os("KEEP_TEST_REATTACH_MOUNT"),
        env::var_os("KEEP_TEST_REATTACH_OCCUPIER"),
        env::var_os("KEEP_TEST_REATTACH_OCCUPIER_MOUNT"),
    ) else {
        return Ok(());
    };
    let (store, writer) = fixture::initialized("darwin-reattach")?;
    drop(writer);
    load(store.path())?;
    let before = fs::metadata(store.path())?;

    hdiutil(&["detach".as_ref(), mount.as_ref()])?;
    attach(&occupier, &occupier_mount)?;
    attach(&image, &mount)?;

    let after = fs::metadata(store.path())?;
    assert_ne!(
        before.dev(),
        after.dev(),
        "the volume kept its device number"
    );
    assert_eq!(before.ino(), after.ino());
    let reopened = load(store.path());
    hdiutil(&["detach".as_ref(), occupier_mount.as_ref()])?;
    reopened
}
