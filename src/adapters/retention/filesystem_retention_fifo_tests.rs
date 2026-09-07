//! Filesystem retention non-regular-file laws: a FIFO at a protocol name refuses, never blocks.
//!
//! Linux only: the fixture creates the FIFO through `mknodat`.

use std::error::Error;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, manifest_pool_path, migrated_store, open_authority,
};
use crate::adapters::FilesystemVersionTwoAdmission;
use crate::execute_retention_publication;

const DEADLINE: Duration = Duration::from_secs(5);

#[test]
fn a_fifo_at_the_retention_head_refuses_instead_of_blocking() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("filesystem-retention-fifo-head")?;
    make_fifo(&sandbox.path().join("retention").join("HEAD"))?;

    let outcome = completes_within(move || authority.observe_current().map(|_| ()))?;

    assert!(outcome.is_err(), "FIFO head was unexpectedly admitted");
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_fifo_at_the_format_marker_refuses_instead_of_blocking() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("filesystem-retention-fifo-marker")?;
    std::fs::remove_file(sandbox.path().join("FORMAT"))?;
    make_fifo(&sandbox.path().join("FORMAT"))?;
    let path = sandbox.path().to_path_buf();

    let outcome = completes_within(move || {
        FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(&path).map(|_| ())
    })?;

    assert!(
        outcome.is_err(),
        "FIFO format marker was unexpectedly admitted"
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_fifo_at_a_manifest_pool_name_refuses_instead_of_blocking() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-fifo-pool")?;
    let root_bytes = fixture(ROOT_HEX)?;
    make_fifo(&manifest_pool_path(
        sandbox.path(),
        &initial_preparation(&root_bytes)?,
    ))?;

    let outcome = completes_within(move || {
        let preparation = initial_preparation(&root_bytes).map_err(|error| error.to_string())?;
        execute_retention_publication(&mut authority, &preparation)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })?;

    assert!(
        outcome.is_err(),
        "FIFO pool target was unexpectedly admitted"
    );
    sandbox.remove()?;
    Ok(())
}

/// Creates a FIFO at `path` for the law under test.
///
/// These laws run only on Linux, where rustix exposes `mknodat`; rustix
/// compiles it out on Apple targets, and spawning `mkfifo(1)` instead is not
/// acceptable scaffolding: a spawned child briefly holds copies of every open
/// descriptor, which kept another law's `flock` alive across its
/// drop-and-reopen. CI runs these laws.
fn make_fifo(path: &Path) -> Result<(), Box<dyn Error>> {
    use rustix::fs::{CWD, FileType, Mode, mknodat};

    mknodat(CWD, path, FileType::Fifo, Mode::RUSR | Mode::WUSR, 0)?;
    Ok(())
}

/// Runs `operation` on its own thread and refuses the test if it does not finish.
fn completes_within<E: Send + 'static>(
    operation: impl FnOnce() -> Result<(), E> + Send + 'static,
) -> Result<Result<(), E>, Box<dyn Error>> {
    let (sender, receiver) = mpsc::channel();
    let _worker = thread::spawn(move || {
        let _ = sender.send(operation());
    });
    receiver
        .recv_timeout(DEADLINE)
        .map_err(|_timeout| "operation blocked past the deadline: a FIFO open hung".into())
}
