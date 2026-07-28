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

fn report(environment: &CapturedEnvironment, scenarios: usize, profiles: usize) -> String {
    let mut report = format!(
        "schema\tkeep.streaming-cas-baseline/v1\n\
         metadata\tgit-commit\t{}\n\
         metadata\tgit-tree\t{}\n\
         metadata\trustc-version\t{}\n\
         metadata\ttarget-triple\t{}\n\
         metadata\tos-description\t{}\n\
         metadata\tcpu-model\t{}\n\
         metadata\tlogical-cpu-count\t{}\n\
         metadata\tbuild-profile\toptimized-release\n\
         threshold\tall-performance-metrics\tunconfigured\t\
         requires-controlled-baseline-history\n",
        environment.commit,
        environment.tree,
        environment.rustc_version,
        environment.target_triple,
        environment.host.os_description,
        environment.host.cpu_model,
        environment.host.logical_cpu_count
    );
    for index in 0..scenarios {
        let _written = writeln!(report, "scenario\t{index}");
    }
    for index in 0..profiles {
        let _written = writeln!(report, "profile\t{index}");
    }
    report
}
