//! Admission and atomic persistence of generated baseline bytes.

use super::BenchmarkBaselineError;
use super::environment::CapturedEnvironment;

/// Immutable report bytes admitted against captured measurement coordinates.
#[must_use]
pub(super) struct AdmittedReport<'a> {
    bytes: &'a [u8],
}

impl<'a> AdmittedReport<'a> {
    pub(super) const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

pub(super) fn validate<'a>(
    bytes: &'a [u8],
    environment: &CapturedEnvironment,
) -> Result<AdmittedReport<'a>, BenchmarkBaselineError> {
    let report = super::report_input::decode(bytes)?;
    if report.contains('\r') || !report.ends_with('\n') {
        return violation("report-line-framing");
    }
    super::metadata_uniqueness::admit(report)?;
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
    super::report_grammar::admit(report)?;
    Ok(AdmittedReport { bytes })
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
