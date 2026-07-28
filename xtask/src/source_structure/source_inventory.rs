//! This module owns deterministic source and executable-candidate inventory.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

use crate::git_inventory::{GitPath, paths as git_paths};

use super::repository_path::RepositoryPath;
use super::source_error::SourceStructureError;
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
    pub(super) executable_candidates: Vec<PathBuf>,
}

pub(super) fn collect(repository_root: &Path) -> Result<SourceInventory, SourceStructureError> {
    let present = git_paths(
        repository_root,
        &PRESENT_PATH_ARGUMENTS,
        "git ls-files present",
    )?;
    let deleted = git_paths(
        repository_root,
        &["ls-files", "-z", "--deleted"],
        "git ls-files deleted",
    )?;
    select(&present, &deleted)
}

pub(super) fn select(
    present: &BTreeSet<GitPath>,
    deleted: &BTreeSet<GitPath>,
) -> Result<SourceInventory, SourceStructureError> {
    let modules = select_source_paths(present, deleted)?;
    let executable_candidates = present
        .difference(deleted)
        .filter(|path| !is_source_candidate(path.as_bytes()))
        .map(admit_inspection_path)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SourceInventory {
        modules,
        executable_candidates,
    })
}

pub(super) fn select_source_paths(
    present: &BTreeSet<GitPath>,
    deleted: &BTreeSet<GitPath>,
) -> Result<Vec<RepositoryPath>, SourceStructureError> {
    present
        .difference(deleted)
        .filter(|path| is_source_candidate(path.as_bytes()))
        .map(admit_source_path)
        .collect()
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

fn admit_inspection_path(path: &GitPath) -> Result<PathBuf, SourceStructureError> {
    let relative = PathBuf::from(OsString::from_vec(path.as_bytes().to_vec()));
    if relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(relative)
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
