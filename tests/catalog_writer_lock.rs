//! Persistent one-writer filesystem lock laws.

#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;

use keep::{FilesystemWriterLock, WriterLockAcquireError, WriterLockAcquirePhase};
use sandbox::TestDirectory;
use support::require_error;

const LOCK_NAME: &str = "writer.lock";
const RETAINED_EVIDENCE: &[u8] = b"lock contents prove nothing";

#[test]
fn one_persistent_lock_excludes_every_second_writer() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_lock("writer-exclusion")?;
    let first = FilesystemWriterLock::try_acquire(sandbox.path())?;
    let error = require_error(
        FilesystemWriterLock::try_acquire(sandbox.path()),
        "a second writer acquired the persistent lock",
    )?;

    assert!(matches!(error, WriterLockAcquireError::Busy));
    drop(first);

    let successor = FilesystemWriterLock::try_acquire(sandbox.path())?;
    drop(successor);
    assert_eq!(fs::read(sandbox.path().join(LOCK_NAME))?, RETAINED_EVIDENCE);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn missing_lock_evidence_is_never_created_by_acquisition() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("writer-lock-missing")?;
    let error = require_error(
        FilesystemWriterLock::try_acquire(sandbox.path()),
        "writer acquisition created a missing lock file",
    )?;

    assert!(matches!(
        error,
        WriterLockAcquireError::Io {
            phase: WriterLockAcquirePhase::OpenFile,
            ref source,
        } if source.kind() == std::io::ErrorKind::NotFound
    ));
    assert!(!sandbox.path().join(LOCK_NAME).exists());
    sandbox.remove()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn lock_acquisition_never_follows_a_symbolic_link() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let sandbox = TestDirectory::create("writer-lock-symlink")?;
    let target = sandbox.path().join("target.lock");
    fs::write(&target, RETAINED_EVIDENCE)?;
    symlink(&target, sandbox.path().join(LOCK_NAME))?;
    let error = require_error(
        FilesystemWriterLock::try_acquire(sandbox.path()),
        "writer acquisition followed a symbolic lock path",
    )?;

    assert!(matches!(
        error,
        WriterLockAcquireError::Io {
            phase: WriterLockAcquirePhase::OpenFile,
            ..
        }
    ));
    assert_eq!(fs::read(target)?, RETAINED_EVIDENCE);
    sandbox.remove()?;
    Ok(())
}

fn initialized_lock(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    fs::write(sandbox.path().join(LOCK_NAME), RETAINED_EVIDENCE)?;
    Ok(sandbox)
}
