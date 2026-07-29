//! This module owns deterministic subprocess durability crash-matrix execution.

mod child;
mod error;
mod process;

use std::ffi::{OsStr, OsString};
use std::path::Path;

pub(crate) use error::DurabilityCrashMatrixError;
use xtask::{
    DurabilityCrashCase, DurabilityCrashOccurrence, DurabilityCrashPoint, DurabilityCrashPosition,
};

const CASE_ARGUMENT: &str = "--case";

pub(crate) fn run(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<(), DurabilityCrashMatrixError> {
    let Some(flag) = arguments.next() else {
        for case in DurabilityCrashCase::all() {
            process::run(repository_root, case)?;
        }
        return Ok(());
    };
    if flag != OsStr::new(CASE_ARGUMENT) {
        return Err(DurabilityCrashMatrixError::Usage);
    }
    let case = parse_case(&mut arguments)?;
    refuse_extra(&mut arguments)?;
    process::run(repository_root, case)
}

pub(crate) fn run_child(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<(), DurabilityCrashMatrixError> {
    let case = parse_case(&mut arguments)?;
    let case_root = arguments.next().ok_or(DurabilityCrashMatrixError::Usage)?;
    let readiness_socket = arguments.next().ok_or(DurabilityCrashMatrixError::Usage)?;
    refuse_extra(&mut arguments)?;
    child::run(case, Path::new(&case_root), Path::new(&readiness_socket))
}

fn parse_case(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<DurabilityCrashCase, DurabilityCrashMatrixError> {
    let point_argument = arguments.next().ok_or(DurabilityCrashMatrixError::Usage)?;
    let point_text = point_argument
        .to_str()
        .ok_or(DurabilityCrashMatrixError::InvalidPointEncoding)?;
    let point = DurabilityCrashPoint::from_identifier(point_text)
        .ok_or_else(|| DurabilityCrashMatrixError::UnknownPoint(point_text.into()))?;
    let position_argument = arguments.next().ok_or(DurabilityCrashMatrixError::Usage)?;
    let position_text = position_argument
        .to_str()
        .ok_or(DurabilityCrashMatrixError::InvalidPositionEncoding)?;
    let position = DurabilityCrashPosition::from_identifier(position_text)
        .ok_or_else(|| DurabilityCrashMatrixError::UnknownPosition(position_text.into()))?;
    let occurrence = point
        .occurrence_counted()
        .then_some(DurabilityCrashOccurrence::FIRST);
    DurabilityCrashCase::new(point, position, occurrence)
        .map_err(DurabilityCrashMatrixError::InvalidCase)
}

fn refuse_extra(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<(), DurabilityCrashMatrixError> {
    if arguments.next().is_some() {
        Err(DurabilityCrashMatrixError::Usage)
    } else {
        Ok(())
    }
}
