//! Deterministic subprocess process-death laws.

use std::error::Error;
use std::process::Command;

#[test]
fn one_selected_case_reaches_readiness_and_survives_process_death() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "durability-crash-matrix",
            "--case",
            "KEEP-CRASH-001",
            "during",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "selected crash case failed: {output:?}"
    );
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
    Ok(())
}
