//! Laws for admission of optimized benchmark subprocess output.

use std::fmt::Write;

use super::artifact;
use super::environment::CapturedEnvironment;
use super::{BenchmarkBaselineError, admit_clean_source, admit_stable_environment};

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
    };
    assert!(matches!(
        artifact::validate(report.as_bytes(), &mismatch),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-git-commit"
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

fn environment() -> CapturedEnvironment {
    CapturedEnvironment {
        commit: String::from("0123456789abcdef0123456789abcdef01234567"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0"),
        target_triple: String::from("aarch64-apple-darwin"),
    }
}

fn report(environment: &CapturedEnvironment, scenarios: usize, profiles: usize) -> String {
    let mut report = format!(
        "schema\tkeep.streaming-cas-baseline/v1\n\
         metadata\tgit-commit\t{}\n\
         metadata\tgit-tree\t{}\n\
         metadata\tbuild-profile\toptimized-release\n\
         threshold\tall-performance-metrics\tunconfigured\t\
         requires-controlled-baseline-history\n",
        environment.commit, environment.tree
    );
    for index in 0..scenarios {
        let _written = writeln!(report, "scenario\t{index}");
    }
    for index in 0..profiles {
        let _written = writeln!(report, "profile\t{index}");
    }
    report
}
