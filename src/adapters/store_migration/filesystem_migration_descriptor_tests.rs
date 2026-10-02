//! This module owns migration admission diagnostics under descriptor exhaustion.

use std::error::Error;
use std::fs::File;
use std::process::Command;

use super::super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::{Error as AuthorityError, FilesystemStoreMigrationAuthority, FilesystemWriterLock};

const CHILD: &str = "KEEP_MIGRATION_DESCRIPTOR_CHILD";
const TEST: &str = "adapters::store_migration::filesystem_migration_repository_tasks::descriptor_tests::a_root_clone_failure_reports_the_namespace_boundary";

// Size: medium. Oracle: capability duplication failure is Namespace with its original EMFILE.
// The isolated child owns a 64-descriptor ceiling and a 20-second execution ceiling.
// Delete when repository-task migration admission is removed or stronger kernel-fault coverage subsumes it.
#[test]
fn a_root_clone_failure_reports_the_namespace_boundary() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(CHILD).is_some() {
        return exhaust_descriptors();
    }
    let output = Command::new("timeout")
        .args([
            "20s",
            "/bin/sh",
            "-c",
            "ulimit -n 64; exec \"$@\"",
            "migration-descriptor-test",
        ])
        .arg(std::env::current_exe()?)
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .output()?;
    assert!(
        output.status.success(),
        "isolated migration clone-failure law failed: {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn exhaust_descriptors() -> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("migration-clone-descriptor-exhaustion")?;
    drop(authority);
    let lock = FilesystemWriterLock::try_acquire(sandbox.path())?;
    let descriptors = fill_descriptors()?;
    let result = FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks(
        lock,
        maximum_policy(),
    );
    drop(descriptors);
    let error = result
        .err()
        .ok_or("descriptor exhaustion unexpectedly admitted migration")?;
    assert!(
        matches!(error, AuthorityError::Namespace { ref source }
        if source.raw_os_error() == Some(rustix::io::Errno::MFILE.raw_os_error())),
        "root clone failure must retain Namespace and EMFILE: {error:?}"
    );
    sandbox.remove()?;
    Ok(())
}

fn fill_descriptors() -> Result<Vec<File>, Box<dyn Error>> {
    let mut descriptors = Vec::with_capacity(64);
    for _ in 0..128 {
        match File::open("/dev/null") {
            Ok(file) => descriptors.push(file),
            Err(error) if error.raw_os_error() == Some(rustix::io::Errno::MFILE.raw_os_error()) => {
                return Ok(descriptors);
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err("the child descriptor ceiling was not enforced".into())
}
