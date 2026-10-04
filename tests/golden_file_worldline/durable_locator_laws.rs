//! A durable handle's relative locator is bound when the handle is created.
//!
//! Size: medium; one isolated child owns working-directory changes and real
//! production-profile stores. Oracle: changing cwd cannot change a handle's
//! selected store. The child has a 20-second execution ceiling, not a latency
//! assertion. Delete if relative locators are removed or stronger laws subsume it.

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use keep::{DurableStore, DurableStoreError, ReaderAttemptLimit};

use super::durable_fixture::{build, identify, policy};
use super::durable_sandbox::TestDirectory;

type TestResult = Result<(), Box<dyn Error>>;

const CHILD: &str = "KEEP_DURABLE_LOCATOR_CHILD";
const LAW: &str = "durable_locator_laws::relative_store_handles_keep_their_initial_store_after_a_directory_change";

#[test]
fn relative_store_handles_keep_their_initial_store_after_a_directory_change()
-> Result<(), Box<dyn Error>> {
    run_isolated(LAW, change_directory_after_open)
}

#[test]
fn an_unresolvable_relative_locator_preserves_its_io_cause() -> Result<(), Box<dyn Error>> {
    run_isolated(
        "durable_locator_laws::an_unresolvable_relative_locator_preserves_its_io_cause",
        deleted_current_directory,
    )
}

fn run_isolated(law: &str, operation: fn() -> TestResult) -> Result<(), Box<dyn Error>> {
    let child_arguments = ["--exact", law, "--nocapture", "--test-threads=1"];
    if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(law))
        && std::env::args_os()
            .skip(1)
            .eq(child_arguments.map(std::ffi::OsStr::new))
    {
        return operation();
    }
    let output = Command::new("timeout")
        .arg("20s")
        .arg(std::env::current_exe()?)
        .args(child_arguments)
        .env(CHILD, law)
        .output()?;
    assert!(
        output.status.success(),
        "isolated locator law failed: {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn deleted_current_directory() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("durable-locator-deleted-cwd")?;
    let restore = WorkingDirectory(std::env::current_dir()?);
    std::env::set_current_dir(sandbox.path())?;
    std::fs::remove_dir(sandbox.path())?;

    let result = DurableStore::open(Path::new("."), policy()?, ReaderAttemptLimit::DEFAULT);

    restore.restore()?;
    let refusal = result
        .err()
        .ok_or("an unresolvable relative locator was accepted")?;
    assert!(
        matches!(&refusal, DurableStoreError::Locator { source }
        if source.kind() == io::ErrorKind::NotFound
            && source.raw_os_error() == Some(rustix::io::Errno::NOENT.raw_os_error())),
        "locator failure must retain its exact NotFound cause: {refusal:?}"
    );
    assert_eq!(
        refusal
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .and_then(io::Error::raw_os_error),
        Some(rustix::io::Errno::NOENT.raw_os_error()),
        "the locator's original I/O cause must remain available through Error::source"
    );
    Ok(())
}

fn change_directory_after_open() -> Result<(), Box<dyn Error>> {
    let first = build("durable-locator-first", &[b"first store"])?;
    let second = build("durable-locator-second", &[b"second store"])?;
    let first_blob = identify(b"first store")?.target;
    let second_blob = identify(b"second store")?.target;
    let restore = WorkingDirectory(std::env::current_dir()?);
    std::env::set_current_dir(first.path())?;
    let store = DurableStore::open(Path::new("."), policy()?, ReaderAttemptLimit::DEFAULT)?;
    assert!(
        store.contains_blob(first_blob)?,
        "initial store must contain its own blob"
    );

    std::env::set_current_dir(second.path())?;

    assert!(
        store.contains_blob(first_blob)?,
        "the same handle must retain its original store after cwd changes"
    );
    assert!(
        !store.contains_blob(second_blob)?,
        "the other store must not supply retention through the original handle"
    );
    let mut output = Vec::new();
    let _receipt = store.reconstruct(first_blob, &mut output)?;
    assert_eq!(
        output, b"first store",
        "relative locator must still read the original store's bytes"
    );
    restore.restore()?;
    Ok(())
}

// The normal path checks restoration; Drop also attempts it on errors/unwind.
// This guard does not make cwd mutation safe in a parallel shared process;
// run_isolated admits only the exact single-law child invocation above.
struct WorkingDirectory(PathBuf);

impl WorkingDirectory {
    fn restore(&self) -> io::Result<()> {
        std::env::set_current_dir(&self.0)
    }
}

impl Drop for WorkingDirectory {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
