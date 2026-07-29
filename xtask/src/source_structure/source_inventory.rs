//! This module owns deterministic source and executable-candidate inventory.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

use crate::git_inventory::{GitPath, paths_with};
use crate::repository_file::RepositoryProcessDirectory;

use super::repository_path::RepositoryPath;
use super::source_error::SourceStructureError;
use super::source_file::{FileExecution, TrackedFileMode};
use super::source_kind::{is_python_module, is_source_candidate};

pub(super) const PRESENT_PATH_ARGUMENTS: [&str; 5] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
];

pub(super) struct SourceInventory {
    pub(super) modules: Vec<RepositoryPath>,
    pub(super) executable_candidates: Vec<InspectionPath>,
    pub(super) tracked_modes: BTreeMap<PathBuf, TrackedFileMode>,
}

/// A repository-relative path admitted for executable inspection.
pub(super) struct InspectionPath(PathBuf);

impl InspectionPath {
    /// Returns the validated platform path.
    pub(super) fn as_path(&self) -> &Path {
        &self.0
    }
}

pub(super) fn collect(
    process_directory: &RepositoryProcessDirectory,
) -> Result<SourceInventory, SourceStructureError> {
    let present = paths_with(&PRESENT_PATH_ARGUMENTS, "git ls-files present", |command| {
        process_directory.spawn(command)
    })?;
    let deleted = paths_with(
        &["ls-files", "-z", "--deleted"],
        "git ls-files deleted",
        |command| process_directory.spawn(command),
    )?;
    let tracked_modes = tracked_modes(process_directory)?;
    let mut inventory = select(&present, &deleted)?;
    inventory.tracked_modes = tracked_modes;
    Ok(inventory)
}

pub(super) fn select(
    present: &BTreeSet<GitPath>,
    deleted: &BTreeSet<GitPath>,
) -> Result<SourceInventory, SourceStructureError> {
    let mut modules = Vec::new();
    let mut executable_candidates = Vec::new();
    for path in present.difference(deleted) {
        if is_source_candidate(path.as_bytes()) {
            modules.push(admit_source_path(path)?);
        } else {
            executable_candidates.push(admit_inspection_path(path)?);
        }
    }
    Ok(SourceInventory {
        modules,
        executable_candidates,
        tracked_modes: BTreeMap::new(),
    })
}

fn tracked_modes(
    process_directory: &RepositoryProcessDirectory,
) -> Result<BTreeMap<PathBuf, TrackedFileMode>, SourceStructureError> {
    let records = paths_with(
        &["ls-files", "-z", "--cached", "--stage"],
        "git ls-files tracked modes",
        |command| process_directory.spawn(command),
    )?;
    let mut modes = BTreeMap::new();
    for record in records {
        let (path, mode) = parse_tracked_mode(&record)?;
        if modes.insert(path, mode).is_some() {
            return Err(SourceStructureError::GitIndexRecord);
        }
    }
    Ok(modes)
}

fn parse_tracked_mode(
    record: &GitPath,
) -> Result<(PathBuf, TrackedFileMode), SourceStructureError> {
    let bytes = record.as_bytes();
    let tab = bytes
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or(SourceStructureError::GitIndexRecord)?;
    let path_start = tab
        .checked_add(1)
        .ok_or(SourceStructureError::GitIndexRecord)?;
    let header = bytes
        .get(..tab)
        .ok_or(SourceStructureError::GitIndexRecord)?;
    let path = bytes
        .get(path_start..)
        .filter(|path| !path.is_empty())
        .ok_or(SourceStructureError::GitIndexRecord)?;
    let mut fields = header.split(|byte| *byte == b' ');
    let mode = fields.next().ok_or(SourceStructureError::GitIndexRecord)?;
    let object = fields.next().ok_or(SourceStructureError::GitIndexRecord)?;
    let stage = fields.next().ok_or(SourceStructureError::GitIndexRecord)?;
    if object.is_empty() || stage != b"0" || fields.next().is_some() {
        return Err(SourceStructureError::GitIndexRecord);
    }
    let mode = match mode {
        b"100644" => TrackedFileMode::Regular(FileExecution::NonExecutable),
        b"100755" => TrackedFileMode::Regular(FileExecution::Executable),
        b"120000" | b"160000" => TrackedFileMode::NonRegular,
        _ => return Err(SourceStructureError::GitIndexRecord),
    };
    let path = admit_inspection_path(&GitPath::new(path.to_vec()))?;
    Ok((path.0, mode))
}

#[cfg(test)]
pub(super) fn select_source_paths(
    present: &BTreeSet<GitPath>,
    deleted: &BTreeSet<GitPath>,
) -> Result<Vec<RepositoryPath>, SourceStructureError> {
    select(present, deleted).map(|inventory| inventory.modules)
}

fn admit_source_path(path: &GitPath) -> Result<RepositoryPath, SourceStructureError> {
    let python = is_python_module(path.as_bytes());
    let text = path_text(path, "source path admission")?;
    let relative = RepositoryPath::admit(text)?;
    if python {
        Err(SourceStructureError::PythonSource(
            relative.as_path().to_owned(),
        ))
    } else {
        Ok(relative)
    }
}

fn admit_inspection_path(path: &GitPath) -> Result<InspectionPath, SourceStructureError> {
    let relative = PathBuf::from(OsString::from_vec(path.as_bytes().to_vec()));
    if relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(InspectionPath(relative))
    } else {
        Err(SourceStructureError::InvalidPath(path_text(
            path,
            "executable path admission",
        )?))
    }
}

fn path_text(path: &GitPath, operation: &'static str) -> Result<String, SourceStructureError> {
    String::from_utf8(path.as_bytes().to_vec())
        .map_err(|source| SourceStructureError::GitPathEncoding { operation, source })
}
