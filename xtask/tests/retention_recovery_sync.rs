//! Linux OS-boundary evidence for recovered retention file durability.

#![cfg(all(feature = "repository-tasks", target_os = "linux"))]

use std::error::Error;
use std::process::Command;

// Size: medium (local subprocesses and filesystem).
// Oracle: retention recovery must sync exact stage contents before publication.
// Delete only when this protocol is removed or stronger OS fault evidence subsumes it.
#[test]
fn recovered_root_contents_are_synchronized_before_pool_publication() -> Result<(), Box<dyn Error>>
{
    require_stage_sync("KEEP-CRASH-036", "root.next", "linkat(")
}

#[test]
fn recovered_manifest_contents_are_synchronized_before_pool_publication()
-> Result<(), Box<dyn Error>> {
    require_stage_sync("KEEP-CRASH-042", "manifest.next", "linkat(")
}

#[test]
fn recovered_head_contents_are_synchronized_before_head_publication() -> Result<(), Box<dyn Error>>
{
    require_stage_sync("KEEP-CRASH-046", "head.next", "renameat(")
}

fn require_stage_sync(point: &str, stage: &str, publication: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("strace")
        .args(["-f", "-yy", "-e", "trace=fsync,linkat,renameat,renameat2"])
        .arg(env!("CARGO_BIN_EXE_xtask"))
        .args(["durability-crash-matrix", "--case", point, "after"])
        .output()?;
    assert!(
        output.status.success(),
        "traced recovery failed: {output:?}"
    );
    let trace = std::str::from_utf8(&output.stderr)?;
    // The killed writer has a [pid ...] prefix. Only the surviving parent
    // performs restart recovery; its unprefixed syscalls are the observation.
    let source = format!("\"{stage}\"");
    let boundary = trace
        .lines()
        .position(|line| line.starts_with(publication) && line.contains(&source))
        .ok_or_else(|| format!("no recovery publication witness for {stage}: {trace}"))?;
    let file = format!("/retention/{stage}>");
    assert!(
        trace.lines().take(boundary).any(|line| {
            line.starts_with("fsync(") && line.contains(&file) && line.ends_with("= 0")
        }),
        "recovery must successfully synchronize {stage} contents before publication: {trace}"
    );
    Ok(())
}
