//! Laws for admission of optimized benchmark subprocess output.

use std::error::Error;
use std::fmt::Write;
use std::fs;
use std::io;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;

use super::artifact;
use super::environment::{self, CapturedEnvironment};
use super::host_environment::CapturedHost;
use super::{BenchmarkBaselineError, admit_clean_source, admit_stable_environment};
use crate::test_directory::TestDirectory;

const BENCHMARK_TASK_SOURCE: &str = include_str!("mod.rs");

#[test]
fn report_admission_binds_release_evidence_to_captured_git_state() {
    let environment = environment();
    let report = report(&environment, 13, 5);

    assert!(artifact::validate(report.as_bytes(), &environment).is_ok());

    let mismatch = CapturedEnvironment {
        commit: String::from("ffffffffffffffffffffffffffffffffffffffff"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0"),
        target_triple: String::from("aarch64-apple-darwin"),
        host: host(),
    };
    assert!(matches!(
        artifact::validate(report.as_bytes(), &mismatch),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-git-commit"
        })
    ));

    let wrong_cpu_count = report.replace(
        "metadata\tlogical-cpu-count\t1",
        "metadata\tlogical-cpu-count\t2",
    );
    assert!(matches!(
        artifact::validate(wrong_cpu_count.as_bytes(), &environment),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-logical-cpu-count"
        })
    ));
}

#[test]
fn report_admission_requires_complete_scenario_and_profile_catalogs() {
    let environment = environment();
    let missing_scenario = report(&environment, 12, 5);
    let missing_profile = report(&environment, 13, 4);

    assert!(matches!(
        artifact::validate(missing_scenario.as_bytes(), &environment),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-scenario-count"
        })
    ));
    assert!(matches!(
        artifact::validate(missing_profile.as_bytes(), &environment),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-profile-count"
        })
    ));
}

#[test]
fn optimized_baselines_refuse_dirty_or_drifting_source_coordinates() {
    let clean = environment();
    let dirty = CapturedEnvironment {
        tree: "dirty",
        ..environment()
    };
    let changed = CapturedEnvironment {
        commit: String::from("ffffffffffffffffffffffffffffffffffffffff"),
        ..environment()
    };

    assert!(matches!(
        admit_clean_source(&dirty),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "benchmark-source-is-dirty"
        })
    ));
    assert!(admit_clean_source(&clean).is_ok());
    assert!(matches!(
        admit_stable_environment(&clean, &changed),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "benchmark-environment-changed-during-run"
        })
    ));
    assert!(admit_stable_environment(&clean, &clean).is_ok());
}

#[test]
fn successful_benchmark_publication_has_no_stdout_boundary() {
    assert!(!BENCHMARK_TASK_SOURCE.contains("std::io::stdout"));
    assert!(!BENCHMARK_TASK_SOURCE.contains("use std::io::Write"));
}

#[test]
fn captured_source_identity_detects_assume_unchanged_bytes() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-hidden-source")?;
    git(directory.path(), &["init", "--quiet"])?;
    git(directory.path(), &["config", "user.name", "Keep Tests"])?;
    git(
        directory.path(),
        &["config", "user.email", "keep-tests@example.invalid"],
    )?;
    let source = directory.path().join("tracked.txt");
    fs::write(&source, b"law\n")?;
    git(directory.path(), &["add", "tracked.txt"])?;
    git(directory.path(), &["commit", "--quiet", "-m", "fixture"])?;
    git(
        directory.path(),
        &["update-index", "--assume-unchanged", "tracked.txt"],
    )?;
    fs::write(&source, b"rot\n")?;

    let status = git(
        directory.path(),
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    assert!(status.is_empty());
    assert_eq!(environment::capture(directory.path())?.tree, "dirty");
    directory.close()?;
    Ok(())
}

fn git(repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(io::Error::other(format!(
            "fixture Git command failed with {:?}",
            output.status.code()
        )))
    }
}

fn environment() -> CapturedEnvironment {
    CapturedEnvironment {
        commit: String::from("0123456789abcdef0123456789abcdef01234567"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0"),
        target_triple: String::from("aarch64-apple-darwin"),
        host: host(),
    }
}

fn host() -> CapturedHost {
    CapturedHost {
        os_description: String::from("Darwin 25.3.0 arm64"),
        cpu_model: String::from("Apple M1 Pro"),
        logical_cpu_count: NonZeroUsize::MIN,
    }
}

const HISTORICAL_BASELINE: &str =
    include_str!("../../../benchmark/baselines/c529c07-aarch64-apple-darwin.tsv");

fn report(environment: &CapturedEnvironment, scenarios: usize, profiles: usize) -> String {
    let admitted_scenarios: Vec<_> = HISTORICAL_BASELINE
        .lines()
        .filter(|line| line.starts_with("scenario\t"))
        .take(scenarios)
        .collect();
    let admitted_profiles: Vec<_> = HISTORICAL_BASELINE
        .lines()
        .filter(|line| line.starts_with("profile\t"))
        .take(profiles)
        .collect();
    let mut output = String::new();
    for line in HISTORICAL_BASELINE.lines() {
        if line.starts_with("scenario\t") && !admitted_scenarios.contains(&line) {
            continue;
        }
        if line.starts_with("profile\t") && !admitted_profiles.contains(&line) {
            continue;
        }
        let _written = writeln!(output, "{line}");
    }
    output
        .replace(
            "c529c07f385b5bcd76a4e57c1987001d496f9135",
            &environment.commit,
        )
        .replace(
            "rustc 1.96.0 (ac68faa20 2026-05-25)",
            &environment.rustc_version,
        )
        .replace(
            "metadata\tlogical-cpu-count\t10",
            "metadata\tlogical-cpu-count\t1",
        )
}

#[test]
fn report_admission_refuses_conflicting_and_identical_source_duplicates() {
    let environment = environment();
    for commit in [
        &environment.commit,
        &String::from("ffffffffffffffffffffffffffffffffffffffff"),
    ] {
        let mut bytes = report(&environment, 13, 5);
        let _written = writeln!(bytes, "metadata\tgit-commit\t{commit}");
        assert!(matches!(
            artifact::validate(bytes.as_bytes(), &environment),
            Err(BenchmarkBaselineError::DuplicateReportMetadata { coordinate })
                if coordinate == "git-commit"
        ));
    }
}

#[test]
fn report_admission_requires_exact_headers_and_canonical_numbers() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    let header = super::report_schema::SCENARIO_HEADER;
    for (malformed, expected, observed) in [
        (
            valid.replace("scenario-header\tname", "scenario-header\tnames"),
            header,
            header.replace("\tname\t", "\tnames\t"),
        ),
        (
            valid.replace(
                "scenario\tcold-ingest\tingest-chunk-and-blob-identity\t100",
                "scenario\tcold-ingest\tingest-chunk-and-blob-identity\t0100",
            ),
            "canonical unsigned decimal",
            String::from("0100"),
        ),
        (
            format!("{valid}unexpected\trow\n"),
            "end of report",
            String::from("unexpected\trow"),
        ),
    ] {
        assert!(matches!(
            artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::InvalidReportRow { expected: actual_expected, observed: actual_observed })
                if actual_expected == expected && actual_observed == observed
        ));
    }
    assert!(artifact::validate(valid.as_bytes(), &environment).is_ok());
}

#[test]
fn malformed_report_rows_cannot_forge_diagnostic_lines() {
    let error = BenchmarkBaselineError::InvalidReportRow {
        expected: "canonical unsigned decimal",
        observed: String::from("1\nforged\tvalue"),
    };
    assert_eq!(
        error.to_string(),
        "benchmark report expected canonical unsigned decimal, observed `1\\nforged\\tvalue`"
    );
}

#[path = "numeric_format_tests.rs"]
mod numeric_format_tests;

#[path = "metadata_policy_tests.rs"]
mod metadata_policy_tests;

#[path = "metric_relation_tests.rs"]
mod metric_relation_tests;

#[path = "counter_width_tests.rs"]
mod counter_width_tests;

#[path = "report_input_tests.rs"]
mod report_input_tests;

#[path = "row_mutation_tests.rs"]
mod row_mutation_tests;

#[path = "report_compatibility_tests.rs"]
mod report_compatibility_tests;

#[path = "catalog_membership_tests.rs"]
mod catalog_membership_tests;
