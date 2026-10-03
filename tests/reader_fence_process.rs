//! Public snapshot fencing across actual reader process death and collection.
#![cfg(all(feature = "repository-tasks", target_os = "linux"))]

#[path = "reader_fence_process/fixture.rs"]
pub mod fixture;
#[path = "reader_fence_process/reader.rs"]
pub mod reader;
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;

use keep::FilesystemWriterLock;
use rustix::{
    fs::{FlockOperation, flock},
    io::Errno,
};
use std::os::unix::fs::MetadataExt;
use std::{error::Error, fs};

// Size: medium. Oracle: KEEP-RETENTION-008: a live snapshot excludes collection;
// SIGKILL releases its shared fence without replacing the persistent lock inode.
// Delete only when this contract disappears or stronger kernel-process evidence subsumes it.
#[test]
fn reader_death_releases_collection_without_replacing_the_fence() -> Result<(), Box<dyn Error>> {
    if reader::is_child() {
        return reader::serve();
    }
    let store = fixture::migrated("reader-death")?;
    let lock_path = store.path().join("reader.lock");
    let before = fs::metadata(&lock_path)?;
    let mut reader = reader::Reader::spawn(
        store.path(),
        "reader_death_releases_collection_without_replacing_the_fence",
    )?;
    reader.await_snapshot()?;
    let writer = FilesystemWriterLock::try_acquire(store.path())?;
    let collector = fs::File::open(&lock_path)?;
    assert_eq!(
        flock(&collector, FlockOperation::NonBlockingLockExclusive),
        Err(Errno::WOULDBLOCK),
        "a live public snapshot must exclude collection"
    );
    reader.kill()?;
    flock(&collector, FlockOperation::NonBlockingLockExclusive)?;
    let after = fs::metadata(&lock_path)?;
    assert_eq!(
        (after.dev(), after.ino(), after.len()),
        (before.dev(), before.ino(), 0),
        "reader death must leave the same empty persistent fence"
    );
    drop(collector);
    drop(writer);
    store.remove()?;
    Ok(())
}

// Size: medium. Oracle: the kernel queues the real snapshot's shared flock while
// cooperating collection owns the exclusive fence, then admits it after release.
// Delete only when collection fencing is removed or stronger schedule evidence subsumes it.
#[test]
fn collection_excludes_a_new_snapshot_until_release() -> Result<(), Box<dyn Error>> {
    if reader::is_child() {
        return reader::serve();
    }
    let store = fixture::migrated("collector-exclusion")?;
    let writer = FilesystemWriterLock::try_acquire(store.path())?;
    let collector = fs::File::open(store.path().join("reader.lock"))?;
    flock(&collector, FlockOperation::NonBlockingLockExclusive)?;
    let mut reader = reader::Reader::spawn(
        store.path(),
        "collection_excludes_a_new_snapshot_until_release",
    )?;
    reader.await_kernel_wait(&collector)?;
    drop(collector);
    drop(writer);
    reader.await_snapshot()?;
    reader.finish()?;
    store.remove()?;
    Ok(())
}
