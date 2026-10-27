//! This module owns real production protocol execution for crash children.

mod control;
pub(super) mod crash_blob;
pub(super) mod fixture;
pub(super) mod initialization;
pub(super) mod populated_store;
mod publication;
mod recovery;
mod recovery_storage;
pub(super) mod retained_catalog;
pub(super) mod retention;
mod retention_storage;
mod segment_stage;
pub(super) mod writer;

use std::error::Error;
use std::fs;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use control::CrashControl;
use xtask::{DurabilityCrashCase, DurabilityCrashSequence};

use super::DurabilityCrashMatrixError;

const STORE_DIRECTORY: &str = "store";

pub(super) fn run(
    case: DurabilityCrashCase,
    case_root: &Path,
    readiness: UnixStream,
) -> Result<(), DurabilityCrashMatrixError> {
    let store_root = create_store_root(case_root)?;
    let mut control = CrashControl::new(case, readiness);
    match case.point().sequence() {
        DurabilityCrashSequence::Segment
        | DurabilityCrashSequence::Catalog
        | DurabilityCrashSequence::Head => {
            publication::run(&store_root, control)?;
        }
        DurabilityCrashSequence::RecoveryDiscard => {
            recovery::run(&store_root, &mut control)?;
        }
        DurabilityCrashSequence::Retention => {
            retention::run(&store_root, &mut control)?;
        }
        DurabilityCrashSequence::Initialization => {
            initialization::run(&store_root, control)?;
        }
        DurabilityCrashSequence::RetainedCatalog => {
            retained_catalog::run(&store_root, control)?;
        }
    }
    Err(DurabilityCrashMatrixError::PointSequenceMismatch {
        point: case.point(),
    })
}

fn create_store_root(case_root: &Path) -> Result<PathBuf, DurabilityCrashMatrixError> {
    let store_root = case_root.join(STORE_DIRECTORY);
    fs::create_dir(&store_root)
        .map_err(|source| DurabilityCrashMatrixError::io("create crash store root", source))?;
    Ok(store_root)
}

pub(in crate::durability_crash_matrix) fn verification(
    phase: &'static str,
    source: impl Error + 'static,
) -> DurabilityCrashMatrixError {
    DurabilityCrashMatrixError::Verification {
        phase,
        source: Box::new(source),
    }
}
