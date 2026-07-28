//! Repository-owned optimized benchmark execution and artifact publication.

mod artifact;
mod build_environment;
mod environment;
mod error;
mod host_environment;
mod process;

use std::io::Write;
use std::path::Path;
use std::process::Command;

pub(crate) use error::BenchmarkBaselineError;

const DIAGNOSTIC_LIMIT: usize = 262_144;
const REPORT_LIMIT: usize = 1_048_576;

pub(crate) fn run(repository_root: &Path) -> Result<(), BenchmarkBaselineError> {
    build_environment::admit(repository_root)?;
    let environment = environment::capture(repository_root)?;
    admit_clean_source(&environment)?;
    let output = process::run(
        Command::new(env!("CARGO"))
            .args([
                "run",
                "--quiet",
                "--release",
                "--locked",
                "--target",
                &environment.target_triple,
                "--package",
                "keep-benchmark",
                "--bin",
                "keep-benchmark-baseline",
            ])
            .current_dir(repository_root)
            .env("KEEP_BENCHMARK_GIT_COMMIT", &environment.commit)
            .env("KEEP_BENCHMARK_GIT_TREE", environment.tree)
            .env("KEEP_BENCHMARK_RUSTC_VERSION", &environment.rustc_version)
            .env("KEEP_BENCHMARK_TARGET_TRIPLE", &environment.target_triple)
            .env(
                "KEEP_BENCHMARK_OS_DESCRIPTION",
                &environment.host.os_description,
            )
            .env("KEEP_BENCHMARK_CPU_MODEL", &environment.host.cpu_model)
            .env(
                "KEEP_BENCHMARK_LOGICAL_CPU_COUNT",
                environment.host.logical_cpu_count.to_string(),
            ),
        "cargo",
        REPORT_LIMIT,
        DIAGNOSTIC_LIMIT,
    )?;
    let observed_after = environment::capture(repository_root)?;
    admit_stable_environment(&environment, &observed_after)?;
    if !output.stderr.is_empty() {
        return Err(BenchmarkBaselineError::ReportViolation {
            reason: "successful-benchmark-wrote-diagnostics",
        });
    }
    artifact::validate(&output.stdout, &environment)?;
    let path = artifact::persist(repository_root, &output.stdout)?;
    let relative = path.strip_prefix(repository_root).map_err(|_source| {
        BenchmarkBaselineError::ReportViolation {
            reason: "report-path-escaped-repository",
        }
    })?;
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", relative.display()).map_err(|source| BenchmarkBaselineError::Io {
        action: "write report path to",
        target: Path::new("stdout").to_path_buf(),
        source,
    })
}

fn admit_clean_source(
    environment: &environment::CapturedEnvironment,
) -> Result<(), BenchmarkBaselineError> {
    if environment.tree == "clean" {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "benchmark-source-is-dirty",
        })
    }
}

fn admit_stable_environment(
    expected: &environment::CapturedEnvironment,
    observed: &environment::CapturedEnvironment,
) -> Result<(), BenchmarkBaselineError> {
    if expected == observed {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "benchmark-environment-changed-during-run",
        })
    }
}

#[cfg(test)]
mod tests;
