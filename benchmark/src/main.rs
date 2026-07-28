//! Optimized streaming CAS baseline executable.

#![deny(warnings)]
#![forbid(unsafe_code)]

use std::env;
use std::io;
use std::num::NonZeroUsize;

use keep_benchmark::{
    BaselineEnvironment, BaselineReport, BuildProfile, HostDescription, MeasurementSettings,
    ReportError, SourceTreeState,
};

const GIT_COMMIT: &str = "KEEP_BENCHMARK_GIT_COMMIT";
const GIT_TREE: &str = "KEEP_BENCHMARK_GIT_TREE";
const CPU_MODEL: &str = "KEEP_BENCHMARK_CPU_MODEL";
const LOGICAL_CPU_COUNT: &str = "KEEP_BENCHMARK_LOGICAL_CPU_COUNT";
const OS_DESCRIPTION: &str = "KEEP_BENCHMARK_OS_DESCRIPTION";
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
    let host = HostDescription::new(
        variable(OS_DESCRIPTION, "os-description")?,
        variable(CPU_MODEL, "cpu-model")?,
        logical_cpu_count(&variable(LOGICAL_CPU_COUNT, "logical-cpu-count")?)?,
    )?;
    BaselineEnvironment::new(git_commit, source_tree, rustc_version, target_triple, host)
}

fn logical_cpu_count(value: &str) -> Result<NonZeroUsize, ReportError> {
    value
        .parse::<usize>()
        .ok()
        .and_then(NonZeroUsize::new)
        .ok_or(ReportError::InvalidEnvironmentField {
            field: "logical-cpu-count",
        })
}

fn variable(name: &'static str, field: &'static str) -> Result<String, ReportError> {
    env::var(name).map_err(|_source| ReportError::InvalidEnvironmentField { field })
}

#[cfg(test)]
mod tests {
    use super::{ReportError, logical_cpu_count};

    #[test]
    fn logical_cpu_count_refuses_zero_and_non_numeric_coordinates() {
        for invalid in ["0", "four", "-1"] {
            assert!(matches!(
                logical_cpu_count(invalid),
                Err(ReportError::InvalidEnvironmentField {
                    field: "logical-cpu-count"
                })
            ));
        }
    }
}
