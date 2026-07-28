//! Optimized streaming CAS baseline executable.

#![deny(warnings)]
#![forbid(unsafe_code)]

use std::env;
use std::io;

use keep_benchmark::{
    BaselineEnvironment, BaselineReport, BuildProfile, MeasurementSettings, ReportError,
    SourceTreeState,
};

const GIT_COMMIT: &str = "KEEP_BENCHMARK_GIT_COMMIT";
const GIT_TREE: &str = "KEEP_BENCHMARK_GIT_TREE";
const RUSTC_VERSION: &str = "KEEP_BENCHMARK_RUSTC_VERSION";
const SAMPLE_COUNT: usize = 100;
const TARGET_TRIPLE: &str = "KEEP_BENCHMARK_TARGET_TRIPLE";
const WARMUP_COUNT: usize = 5;

fn main() -> Result<(), ReportError> {
    BuildProfile::current().require_optimized()?;
    let environment = environment()?;
    let settings = MeasurementSettings::new(SAMPLE_COUNT, WARMUP_COUNT)?;
    let report = BaselineReport::collect(environment, settings)?;
    report.write_tsv(&mut io::stdout().lock())
}

fn environment() -> Result<BaselineEnvironment, ReportError> {
    let git_commit = variable(GIT_COMMIT, "git-commit")?;
    let source_tree = match variable(GIT_TREE, "git-tree")?.as_str() {
        "clean" => SourceTreeState::Clean,
        "dirty" => SourceTreeState::Dirty,
        _ => {
            return Err(ReportError::InvalidEnvironmentField { field: "git-tree" });
        }
    };
    let rustc_version = variable(RUSTC_VERSION, "rustc-version")?;
    let target_triple = variable(TARGET_TRIPLE, "target-triple")?;
    let logical_cpu_count =
        std::thread::available_parallelism().map_err(|source| ReportError::Environment {
            action: "read logical CPU count from",
            source,
        })?;
    BaselineEnvironment::new(
        git_commit,
        source_tree,
        rustc_version,
        target_triple,
        logical_cpu_count,
    )
}

fn variable(name: &'static str, field: &'static str) -> Result<String, ReportError> {
    env::var(name).map_err(|_source| ReportError::InvalidEnvironmentField { field })
}
