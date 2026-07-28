use std::path::Path;

use crate::repository_file::RepositoryRoot;

const WORKFLOW: &str = r"name: CI
jobs:
  documentation:
    name: Documentation
    steps:
      - name: Install Rust
        run: rustup show
      - name: Verify
        run: cargo xtask documentation-integrity-check
  next-job:
    steps: []
";

#[test]
fn documentation_job_delegates_once_to_the_rust_boundary() {
    assert!(super::admit(WORKFLOW).is_ok());
}

#[test]
fn documentation_job_refuses_python_execution() {
    let workflow = WORKFLOW.replace(
        "  next-job:",
        "      - name: Legacy checker\n        run: python3 scripts/check_markdown.py\n  next-job:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job contains no Python execution",
        })
    ));
}

#[test]
fn committed_documentation_job_uses_the_rust_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    let repository_root = RepositoryRoot::open(root)?;
    super::check(&repository_root)?;
    assert!(repository_root.is_current_path()?);
    Ok(())
}
