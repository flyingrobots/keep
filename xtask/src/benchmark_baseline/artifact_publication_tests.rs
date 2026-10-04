//! Recovery and exclusion laws for benchmark artifact publication.

use std::error::Error;
use std::fs::{self, OpenOptions};

use super::{LOCK_NAME, OUTPUT_RELATIVE_PATH, STAGE_NAME, persist};
use crate::benchmark_baseline::BenchmarkBaselineError;
use crate::test_directory::TestDirectory;

#[test]
fn interrupted_baseline_stage_is_recovered_before_publication() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-stage-recovery")?;
    let (output, parent) = output_paths(&directory)?;
    let stage = parent.join(STAGE_NAME);
    fs::write(&stage, b"abandoned")?;

    let (bytes, environment) = report_fixture()?;
    let report = super::super::artifact::validate(bytes.as_bytes(), &environment)?;
    persist(directory.path(), &report)?;

    assert_eq!(fs::read(&output)?, bytes.as_bytes());
    assert!(!stage.exists());
    directory.close()?;
    Ok(())
}

#[test]
fn failed_baseline_publication_removes_its_stage() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-stage-failure")?;
    let (output, parent) = output_paths(&directory)?;
    fs::create_dir(&output)?;

    let (bytes, environment) = report_fixture()?;
    let report = super::super::artifact::validate(bytes.as_bytes(), &environment)?;
    assert!(matches!(
        persist(directory.path(), &report),
        Err(BenchmarkBaselineError::Io {
            action: "publish report",
            ..
        })
    ));
    assert!(!parent.join(STAGE_NAME).exists());
    directory.close()?;
    Ok(())
}

#[test]
fn concurrent_baseline_publishers_are_refused() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-stage-lock")?;
    let (_output, parent) = output_paths(&directory)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join(LOCK_NAME))?;
    lock.try_lock()?;

    let (bytes, environment) = report_fixture()?;
    let report = super::super::artifact::validate(bytes.as_bytes(), &environment)?;
    assert!(matches!(
        persist(directory.path(), &report),
        Err(BenchmarkBaselineError::ReportViolation {
            reason: "benchmark-publication-already-active"
        })
    ));
    drop(lock);
    directory.close()?;
    Ok(())
}

fn output_paths(
    directory: &TestDirectory,
) -> Result<(std::path::PathBuf, std::path::PathBuf), Box<dyn Error>> {
    let output = directory.path().join(OUTPUT_RELATIVE_PATH);
    let parent = output.parent().ok_or("report has no parent")?.to_path_buf();
    fs::create_dir_all(&parent)?;
    Ok((output, parent))
}

#[test]
fn refused_report_preserves_prior_artifact_and_interrupted_stage() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-admission-preservation")?;
    let (output, parent) = output_paths(&directory)?;
    fs::write(&output, b"previous artifact\n")?;
    let stage = parent.join(STAGE_NAME);
    fs::write(&stage, b"interrupted stage\n")?;
    let (bytes, environment) = report_fixture()?;
    let malformed =
        format!("{bytes}metadata\tgit-commit\tffffffffffffffffffffffffffffffffffffffff\n");
    assert!(
        matches!(super::super::artifact::validate(malformed.as_bytes(), &environment)
            .and_then(|report| persist(directory.path(), &report)),
        Err(BenchmarkBaselineError::DuplicateReportMetadata { coordinate })
            if coordinate == "git-commit")
    );
    assert_eq!(fs::read(&output)?, b"previous artifact\n");
    assert_eq!(fs::read(&stage)?, b"interrupted stage\n");
    assert!(!parent.join(LOCK_NAME).exists());
    directory.close()?;
    Ok(())
}

fn report_fixture() -> Result<
    (
        String,
        crate::benchmark_baseline::environment::CapturedEnvironment,
    ),
    Box<dyn Error>,
> {
    use crate::benchmark_baseline::environment::CapturedEnvironment;
    use crate::benchmark_baseline::host_environment::CapturedHost;
    let environment = CapturedEnvironment {
        commit: String::from("c529c07f385b5bcd76a4e57c1987001d496f9135"),
        tree: "clean",
        rustc_version: String::from("rustc 1.96.0 (ac68faa20 2026-05-25)"),
        target_triple: String::from("aarch64-apple-darwin"),
        host: CapturedHost {
            os_description: String::from("Darwin 25.3.0 arm64"),
            cpu_model: String::from("Apple M1 Pro"),
            logical_cpu_count: std::num::NonZeroUsize::new(10).ok_or("invalid CPU fixture")?,
        },
    };
    Ok((
        String::from(include_str!(
            "../../../benchmark/baselines/c529c07-aarch64-apple-darwin.tsv"
        )),
        environment,
    ))
}
