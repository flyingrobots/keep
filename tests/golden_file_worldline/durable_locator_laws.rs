//! A durable handle's relative locator is bound when the handle is created.
//!
//! Size: medium; one isolated child owns working-directory changes and real
//! production-profile stores. Oracle: changing cwd cannot change a handle's
//! selected store. The child has a 20-second execution ceiling, not a latency
//! assertion. Delete if relative locators are removed or stronger laws subsume it.

use std::error::Error;
use std::path::Path;
use std::process::Command;

use keep::{DurableStore, ReaderAttemptLimit};

use super::durable_fixture::{build, identify, policy};

const CHILD: &str = "KEEP_DURABLE_LOCATOR_CHILD";
const LAW: &str = "suite::durable_locator_laws::relative_store_handles_keep_their_initial_store_after_a_directory_change";

#[test]
fn relative_store_handles_keep_their_initial_store_after_a_directory_change()
-> Result<(), Box<dyn Error>> {
    if std::env::var_os(CHILD).is_some() {
        return change_directory_after_open();
    }
    let output = Command::new("timeout")
        .arg("20s")
        .arg(std::env::current_exe()?)
        .args(["--exact", LAW, "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
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

fn change_directory_after_open() -> Result<(), Box<dyn Error>> {
    let first = build("durable-locator-first", &[b"first store"])?;
    let second = build("durable-locator-second", &[b"second store"])?;
    let first_blob = identify(b"first store")?.target;
    let second_blob = identify(b"second store")?.target;
    let original = std::env::current_dir()?;
    std::env::set_current_dir(first.path())?;
    let store = DurableStore::open(Path::new("."), policy()?, ReaderAttemptLimit::DEFAULT);
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
    std::env::set_current_dir(original)?;
    Ok(())
}
