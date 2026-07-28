//! Admission and atomic persistence of generated baseline bytes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::BenchmarkBaselineError;
use super::environment::CapturedEnvironment;

const REPORT_RELATIVE_PATH: &str = "target/benchmark/streaming-cas-baseline-v1.tsv";

pub(super) fn validate(
    bytes: &[u8],
    environment: &CapturedEnvironment,
) -> Result<(), BenchmarkBaselineError> {
    let report =
        std::str::from_utf8(bytes).map_err(|_source| BenchmarkBaselineError::ReportViolation {
            reason: "report-is-not-utf8",
        })?;
    if report.contains('\r') || !report.ends_with('\n') {
        return violation("report-line-framing");
    }
    let mut lines = report.lines();
    if lines.next() != Some("schema\tkeep.streaming-cas-baseline/v1") {
        return violation("report-schema");
    }
    require_line(
        report,
        &format!("metadata\tgit-commit\t{}", environment.commit),
        "report-git-commit",
    )?;
    require_line(
        report,
        &format!("metadata\tgit-tree\t{}", environment.tree),
        "report-git-tree",
    )?;
    require_line(
        report,
        &format!("metadata\trustc-version\t{}", environment.rustc_version),
        "report-rustc-version",
    )?;
    require_line(
        report,
        &format!("metadata\ttarget-triple\t{}", environment.target_triple),
        "report-target-triple",
    )?;
    require_line(
        report,
        &format!(
            "metadata\tos-description\t{}",
            environment.host.os_description
        ),
        "report-os-description",
    )?;
    require_line(
        report,
        &format!("metadata\tcpu-model\t{}", environment.host.cpu_model),
        "report-cpu-model",
    )?;
    require_line(
        report,
        &format!(
            "metadata\tlogical-cpu-count\t{}",
            environment.host.logical_cpu_count
        ),
        "report-logical-cpu-count",
    )?;
    require_line(
        report,
        "metadata\tbuild-profile\toptimized-release",
        "report-build-profile",
    )?;
    require_line(
        report,
        "threshold\tall-performance-metrics\tunconfigured\t\
         requires-controlled-baseline-history",
        "report-threshold-policy",
    )?;
    if report
        .lines()
        .filter(|line| line.starts_with("scenario\t"))
        .count()
        != 13
    {
        return violation("report-scenario-count");
    }
    if report
        .lines()
        .filter(|line| line.starts_with("profile\t"))
        .count()
        != 5
    {
        return violation("report-profile-count");
    }
    Ok(())
}

pub(super) fn persist(
    repository_root: &Path,
    bytes: &[u8],
) -> Result<PathBuf, BenchmarkBaselineError> {
    let output = repository_root.join(REPORT_RELATIVE_PATH);
    let parent = output
        .parent()
        .ok_or(BenchmarkBaselineError::ReportViolation {
            reason: "report-output-has-no-parent",
        })?;
    fs::create_dir_all(parent).map_err(|source| io_error("create directory", parent, source))?;
    let temporary = parent.join(format!(
        ".streaming-cas-baseline-v1.tsv.tmp-{}",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| io_error("create temporary report", &temporary, source))?;
    file.write_all(bytes)
        .map_err(|source| io_error("write temporary report", &temporary, source))?;
    file.sync_all()
        .map_err(|source| io_error("sync temporary report", &temporary, source))?;
    fs::rename(&temporary, &output)
        .map_err(|source| io_error("publish report", &output, source))?;
    Ok(output)
}

fn require_line(
    report: &str,
    expected: &str,
    reason: &'static str,
) -> Result<(), BenchmarkBaselineError> {
    if report.lines().any(|line| line == expected) {
        Ok(())
    } else {
        violation(reason)
    }
}

const fn violation<T>(reason: &'static str) -> Result<T, BenchmarkBaselineError> {
    Err(BenchmarkBaselineError::ReportViolation { reason })
}

fn io_error(action: &'static str, target: &Path, source: std::io::Error) -> BenchmarkBaselineError {
    BenchmarkBaselineError::Io {
        action,
        target: target.to_path_buf(),
        source,
    }
}
