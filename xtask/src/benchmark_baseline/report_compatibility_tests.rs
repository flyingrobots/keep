//! Compatibility laws for historical two-pass and optimized single-pass reports.

use std::num::NonZeroUsize;

use super::super::captured_environment::{CapturedEnvironment, CapturedHost};
use super::super::{BenchmarkBaselineError, artifact};

#[test]
fn single_pass_report_retains_its_source_and_counter_semantics()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = include_bytes!("fixtures/single-pass-report-v1.tsv");
    let environment = CapturedEnvironment {
        commit: String::from("30ffe90e53c01a24d8931244a7f76eaecd0da8a4"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0 (ac68faa20 2026-05-25)"),
        target_triple: String::from("aarch64-apple-darwin"),
        host: CapturedHost {
            os_description: String::from("Darwin 27.0.0 arm64"),
            cpu_model: String::from("Apple M5 Pro"),
            logical_cpu_count: NonZeroUsize::new(18).ok_or("invalid CPU fixture")?,
        },
    };
    let admitted = artifact::validate(bytes, &environment)?;
    assert_eq!(admitted.bytes(), bytes);
    let mismatched = CapturedEnvironment {
        commit: String::from("c529c07f385b5bcd76a4e57c1987001d496f9135"),
        ..environment
    };
    assert!(matches!(
        artifact::validate(bytes, &mismatched),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "report-git-commit"
        })
    ));
    Ok(())
}
