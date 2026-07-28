//! Single-writer recovery and atomic publication of benchmark evidence.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::BenchmarkBaselineError;

const LOCK_NAME: &str = ".streaming-cas-baseline-v1.lock";
const OUTPUT_RELATIVE_PATH: &str = "target/benchmark/streaming-cas-baseline-v1.tsv";
const STAGE_NAME: &str = ".streaming-cas-baseline-v1.tsv.stage";

pub(super) fn persist(repository_root: &Path, bytes: &[u8]) -> Result<(), BenchmarkBaselineError> {
    let output = repository_root.join(OUTPUT_RELATIVE_PATH);
    let parent = output
        .parent()
        .ok_or(BenchmarkBaselineError::ReportViolation {
            reason: "report-output-has-no-parent",
        })?;
    fs::create_dir_all(parent).map_err(|source| io_error("create directory", parent, source))?;
    let publication_lock = PublicationLock::acquire(parent)?;
    let stage_path = parent.join(STAGE_NAME);
    recover_stage(&stage_path)?;
    let mut stage = PublicationStage::new(stage_path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(stage.path())
        .map_err(|source| io_error("create temporary report", stage.path(), source))?;
    file.write_all(bytes)
        .map_err(|source| io_error("write temporary report", stage.path(), source))?;
    file.sync_all()
        .map_err(|source| io_error("sync temporary report", stage.path(), source))?;
    drop(file);
    fs::rename(stage.path(), &output)
        .map_err(|source| io_error("publish report", &output, source))?;
    stage.published();
    drop(publication_lock);
    Ok(())
}

fn recover_stage(path: &Path) -> Result<(), BenchmarkBaselineError> {
    match fs::symlink_metadata(path) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error("inspect temporary report", path, source)),
        Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {
            fs::remove_file(path)
                .map_err(|source| io_error("remove stale temporary report", path, source))
        }
        Ok(_metadata) => Err(BenchmarkBaselineError::ReportViolation {
            reason: "temporary-report-is-not-removable",
        }),
    }
}

struct PublicationLock {
    _file: File,
}

impl PublicationLock {
    fn acquire(parent: &Path) -> Result<Self, BenchmarkBaselineError> {
        let path = parent.join(LOCK_NAME);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| io_error("open benchmark publication lock", &path, source))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(BenchmarkBaselineError::ReportViolation {
                reason: "benchmark-publication-already-active",
            }),
            Err(TryLockError::Error(source)) => Err(io_error(
                "acquire benchmark publication lock",
                &path,
                source,
            )),
        }
    }
}

struct PublicationStage {
    path: PathBuf,
    active: bool,
}

impl PublicationStage {
    const fn new(path: PathBuf) -> Self {
        Self { path, active: true }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    const fn published(&mut self) {
        self.active = false;
    }
}

impl Drop for PublicationStage {
    fn drop(&mut self) {
        if self.active {
            drop(fs::remove_file(&self.path));
        }
    }
}

fn io_error(action: &'static str, target: &Path, source: io::Error) -> BenchmarkBaselineError {
    BenchmarkBaselineError::Io {
        action,
        target: target.to_path_buf(),
        source,
    }
}

#[cfg(test)]
#[path = "artifact_publication_tests.rs"]
mod tests;
