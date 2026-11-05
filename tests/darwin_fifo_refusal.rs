//! A FIFO at a protocol name refuses instead of blocking the reader on Darwin.
//!
//! Size: medium. Oracle: the Linux FIFO laws create FIFOs with `mknodat`, which
//! Rustix omits on Apple targets. Spawning `mkfifo(1)` can leak flock holders to
//! concurrent laws until exec, so this law owns its own test executable.
//! Delete when a safe in-process FIFO constructor reaches the pinned toolchain.

#![cfg(all(feature = "repository-tasks", target_os = "macos"))]

#[allow(dead_code, reason = "this law uses only the initialized-store fixture")]
#[path = "reader_fence_process/fixture.rs"]
pub mod fixture;
#[allow(
    dead_code,
    reason = "this uses only the fixture's subset of the builder"
)]
#[path = "support/populated_store.rs"]
pub mod populated_store;
#[allow(dead_code, reason = "this law uses Drop cleanup")]
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;

use std::{error::Error, fs, process::Command, sync::mpsc, thread, time::Duration};

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot, ReaderAttemptLimit,
    SegmentReadPolicy,
};

const DEADLINE: Duration = Duration::from_secs(20);

#[test]
fn a_fifo_at_the_head_refuses_without_blocking_the_reader() -> Result<(), Box<dyn Error>> {
    let (store, writer) = fixture::initialized("darwin-fifo-head")?;
    drop(writer);
    let head = store.path().join("HEAD");
    fs::remove_file(&head)?;
    assert!(
        Command::new("/usr/bin/mkfifo")
            .arg(&head)
            .status()?
            .success()
    );
    let root = store.path().to_owned();
    let (sender, receiver) = mpsc::channel();
    let _reader = thread::spawn(move || {
        let policy = CatalogRestartByteLimit::new(1_048_576)
            .map(|limit| CatalogRestartPolicy::new(SegmentReadPolicy::MAXIMUM, limit));
        let refused = policy.is_ok_and(|policy| {
            FilesystemRetentionSnapshot::load(
                &root,
                policy,
                ReaderAttemptLimit::DEFAULT,
                keep::StoreExpectation::NoExpectation,
            )
            .is_err()
        });
        let _ = sender.send(refused);
    });
    assert!(receiver.recv_timeout(DEADLINE)?, "a FIFO head was admitted");
    Ok(())
}
