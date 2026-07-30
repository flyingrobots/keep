//! This module owns the crash child readiness boundary.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

use super::DurabilityCrashMatrixError;
use super::production_protocol;
use xtask::DurabilityCrashCase;

pub(super) fn run(
    case: DurabilityCrashCase,
    case_root: &Path,
    readiness_socket: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
    write_marker(case, case_root)?;
    let stream = UnixStream::connect(readiness_socket)
        .map_err(|source| DurabilityCrashMatrixError::io("connect readiness socket", source))?;
    production_protocol::run(case, case_root, stream)
}

fn write_marker(
    case: DurabilityCrashCase,
    case_root: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
    let path = case_root.join("prepared-case");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| DurabilityCrashMatrixError::io("create crash-case marker", source))?;
    file.write_all(&marker(case))
        .map_err(|source| DurabilityCrashMatrixError::io("write crash-case marker", source))?;
    file.sync_all()
        .map_err(|source| DurabilityCrashMatrixError::io("synchronize crash-case marker", source))
}

pub(super) fn marker(case: DurabilityCrashCase) -> Vec<u8> {
    format!(
        "{}\t{}\n",
        case.point().identifier(),
        case.position().identifier()
    )
    .into_bytes()
}
