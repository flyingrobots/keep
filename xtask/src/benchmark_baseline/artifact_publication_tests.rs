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

    persist(directory.path(), b"exact report\n")?;

    assert_eq!(fs::read(&output)?, b"exact report\n");
    assert!(!stage.exists());
    directory.close()?;
    Ok(())
}

#[test]
fn failed_baseline_publication_removes_its_stage() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("benchmark-stage-failure")?;
    let (output, parent) = output_paths(&directory)?;
    fs::create_dir(&output)?;

    assert!(matches!(
        persist(directory.path(), b"exact report\n"),
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

    assert!(matches!(
        persist(directory.path(), b"exact report\n"),
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
