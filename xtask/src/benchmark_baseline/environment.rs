//! Git and compiler coordinates passed into the release benchmark.

use std::path::Path;
use std::process::Command;

use super::BenchmarkBaselineError;
use super::process::{ProcessOutput, run};

const DIAGNOSTIC_LIMIT: usize = 65_536;
const VALUE_LIMIT: usize = 4_096;

pub(super) struct CapturedEnvironment {
    pub(super) commit: String,
    pub(super) tree: &'static str,
    pub(super) rustc_version: String,
    pub(super) target_triple: String,
}

pub(super) fn capture(
    repository_root: &Path,
) -> Result<CapturedEnvironment, BenchmarkBaselineError> {
    let commit = text(
        git(repository_root, &["rev-parse", "--verify", "HEAD"])?,
        "git-commit",
    )?;
    let status = git(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=all", "-z"],
    )?;
    require_silent(&status)?;
    let tree = if status.stdout.is_empty() {
        "clean"
    } else {
        "dirty"
    };
    let rustc_version = text(rustc(&["--version"])?, "rustc-version")?;
    let target_triple = text(rustc(&["--print", "host-tuple"])?, "target-triple")?;
    Ok(CapturedEnvironment {
        commit,
        tree,
        rustc_version,
        target_triple,
    })
}

fn git(
    repository_root: &Path,
    arguments: &[&str],
) -> Result<ProcessOutput, BenchmarkBaselineError> {
    run(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(arguments),
        "git",
        VALUE_LIMIT,
        DIAGNOSTIC_LIMIT,
    )
}

fn rustc(arguments: &[&str]) -> Result<ProcessOutput, BenchmarkBaselineError> {
    run(
        Command::new("rustc").args(arguments),
        "rustc",
        VALUE_LIMIT,
        DIAGNOSTIC_LIMIT,
    )
}

fn text(output: ProcessOutput, coordinate: &'static str) -> Result<String, BenchmarkBaselineError> {
    require_silent(&output)?;
    let text = String::from_utf8(output.stdout)
        .map_err(|source| BenchmarkBaselineError::ValueEncoding { coordinate, source })?;
    let value = text.strip_suffix('\n').unwrap_or(&text);
    if value.is_empty()
        || value.len() > VALUE_LIMIT
        || value.chars().any(char::is_control)
        || value.contains('\t')
    {
        return Err(BenchmarkBaselineError::InvalidValue { coordinate });
    }
    Ok(String::from(value))
}

const fn require_silent(output: &ProcessOutput) -> Result<(), BenchmarkBaselineError> {
    if output.stderr.is_empty() {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "successful-environment-command-wrote-diagnostics",
        })
    }
}
