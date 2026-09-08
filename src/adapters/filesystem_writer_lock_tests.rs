//! Replacement-race laws for persistent writer authority.

use std::error::Error;
use std::fs;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{FileIdentity, LOCK_FILE_NAME, open_existing, verify_current_identity};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{WriterLockAcquireError, WriterLockAcquirePhase};

#[test]
fn replaced_lock_entry_cannot_authorize_the_opened_handle() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("writer-lock-identity")?;
    fs::write(sandbox.path().join(LOCK_FILE_NAME), [])?;
    let directory = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    let opened = open_existing(&directory)?;
    let expected = FileIdentity::read(&opened)?;

    fs::rename(
        sandbox.path().join(LOCK_FILE_NAME),
        sandbox.path().join("displaced.lock"),
    )?;
    fs::write(sandbox.path().join(LOCK_FILE_NAME), [])?;

    let error = verify_current_identity(&directory, expected)
        .err()
        .ok_or("replacement was admitted as the opened lock file")?;
    assert!(matches!(
        error,
        WriterLockAcquireError::Io {
            phase: WriterLockAcquirePhase::VerifyFileIdentity,
            ref source,
        } if source.kind() == std::io::ErrorKind::InvalidData
    ));
    drop(opened);
    sandbox.remove()?;
    Ok(())
}
