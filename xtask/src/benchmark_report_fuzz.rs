//! This module owns the I/O-free production benchmark-report fuzz facade.

#[path = "benchmark_baseline/artifact.rs"]
mod artifact;
#[path = "benchmark_baseline/captured_environment.rs"]
mod captured_environment;
#[path = "benchmark_baseline/counter_widths.rs"]
mod counter_widths;
#[allow(
    dead_code,
    reason = "pure admission reuses the complete task error boundary"
)]
#[path = "benchmark_baseline/error.rs"]
mod error;
#[path = "benchmark_baseline/metadata_policy.rs"]
mod metadata_policy;
#[path = "benchmark_baseline/metadata_uniqueness.rs"]
mod metadata_uniqueness;
#[path = "benchmark_baseline/metric_error.rs"]
mod metric_error;
#[path = "benchmark_baseline/metric_relations.rs"]
mod metric_relations;
#[path = "benchmark_baseline/report_grammar.rs"]
mod report_grammar;
#[path = "benchmark_baseline/report_input.rs"]
mod report_input;
#[path = "benchmark_baseline/report_schema.rs"]
mod report_schema;

use captured_environment::{CapturedEnvironment, CapturedHost};
use error::BenchmarkBaselineError;

mod environment {
    pub(super) use super::captured_environment::CapturedEnvironment;
}

pub(super) fn admit(input: &[u8]) -> Result<(), BenchmarkBaselineError> {
    let environment = CapturedEnvironment {
        commit: String::from("c529c07f385b5bcd76a4e57c1987001d496f9135"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0 (ac68faa20 2026-05-25)"),
        target_triple: String::from("aarch64-apple-darwin"),
        host: CapturedHost {
            os_description: String::from("Darwin 25.3.0 arm64"),
            cpu_model: String::from("Apple M1 Pro"),
            logical_cpu_count: std::num::NonZeroUsize::MIN.saturating_add(9),
        },
    };
    artifact::validate(input, &environment).map(|report| {
        let _admitted_bytes = report.bytes();
    })
}
