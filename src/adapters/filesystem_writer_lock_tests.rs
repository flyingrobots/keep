//! Replacement-race laws for persistent writer authority.

use std::error::Error;
use std::fs;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{FilesystemWriterLock, LOCK_FILE_NAME, acquire_root, open_existing};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{WriterLockAcquireError, WriterLockAcquirePhase};

// Size: medium. Oracle: KEEP-RECOVERY-004 forbids returning mismatched authority.
// This shared acquisition boundary performs the real kernel lock and returns the guard.
// Delete if a stronger scheduled public acquisition law subsumes this transition.
#[test]
fn replaced_lock_entry_refuses_authority_after_kernel_acquisition() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("writer-lock-identity")?;
    fs::write(sandbox.path().join(LOCK_FILE_NAME), b"original evidence")?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let root_lock = acquire_root(&directory)?;
    let opened = open_existing(&directory)?;

    fs::rename(
        sandbox.path().join(LOCK_FILE_NAME),
        sandbox.path().join("displaced.lock"),
    )?;
    fs::write(sandbox.path().join(LOCK_FILE_NAME), b"replacement evidence")?;

    let error = FilesystemWriterLock::acquire(directory, root_lock, opened)
        .err()
        .ok_or("replacement received writer authority")?;
    assert!(
        matches!(
            error,
            WriterLockAcquireError::Io {
                phase: WriterLockAcquirePhase::VerifyFileIdentity,
                ref source,
            } if source.kind() == std::io::ErrorKind::InvalidData
        ),
        "replacement must retain the identity-refusal boundary: {error:?}"
    );
    assert_eq!(
        fs::read(sandbox.path().join("displaced.lock"))?,
        b"original evidence"
    );
    assert_eq!(
        fs::read(sandbox.path().join(LOCK_FILE_NAME))?,
        b"replacement evidence"
    );
    sandbox.remove()?;
    Ok(())
}
