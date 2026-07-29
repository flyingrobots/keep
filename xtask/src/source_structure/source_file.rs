//! This module owns one-handle admission of a repository source file.

use std::fs::{File, Metadata};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::repository_file::{OpenRepositoryFileError, RepositoryFileIdentity, RepositoryRoot};

use super::SourceStructureError;
use super::python_source::executable_uses_python;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FileExecution {
    Executable,
    NonExecutable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TrackedFileMode {
    NonRegular,
    Regular(FileExecution),
}

pub(super) enum SourceFileAdmission {
    Regular(AdmittedSource),
    NonRegular,
}

pub(super) struct AdmittedSource {
    execution: FileExecution,
    file: File,
    identity: RepositoryFileIdentity,
    path: PathBuf,
    relative: PathBuf,
}

impl AdmittedSource {
    pub(super) fn admit(
        source_root: &RepositoryRoot,
        relative: &Path,
        tracked_execution: Option<TrackedFileMode>,
    ) -> Result<SourceFileAdmission, SourceStructureError> {
        let path = source_root.display_path(relative);
        let file = match source_root.open_file(relative) {
            Ok(file) => file,
            Err(OpenRepositoryFileError::NonRegular) => {
                if let Some(TrackedFileMode::Regular(execution)) = tracked_execution {
                    return Err(mode_changed(&path, execution.label(), "nonregular"));
                }
                return Ok(SourceFileAdmission::NonRegular);
            }
            Err(OpenRepositoryFileError::Io(source)) => {
                return Err(SourceStructureError::Inspect { path, source });
            }
        };
        let metadata = file
            .metadata()
            .map_err(|source| SourceStructureError::Inspect {
                path: path.clone(),
                source,
            })?;
        let execution = file_execution(&metadata);
        admit_execution(&path, tracked_execution, execution)?;
        let python = execution == FileExecution::Executable
            && executable_uses_python(&file).map_err(|source| SourceStructureError::Inspect {
                path: path.clone(),
                source,
            })?;
        let source = Self {
            execution,
            file,
            identity: RepositoryFileIdentity::from(&metadata),
            path,
            relative: relative.to_owned(),
        };
        source.verify_current(source_root)?;
        if python {
            return Err(SourceStructureError::PythonSource(relative.to_owned()));
        }
        Ok(SourceFileAdmission::Regular(source))
    }

    pub(super) const fn execution(&self) -> FileExecution {
        self.execution
    }

    pub(super) const fn file(&self) -> &File {
        &self.file
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn verify_current(
        &self,
        source_root: &RepositoryRoot,
    ) -> Result<(), SourceStructureError> {
        let observed = RepositoryFileIdentity::read(&self.file).map_err(|source| {
            SourceStructureError::Inspect {
                path: self.path.clone(),
                source,
            }
        })?;
        let current = match source_root.open_file(&self.relative) {
            Ok(file) => RepositoryFileIdentity::read(&file),
            Err(OpenRepositoryFileError::Io(source))
                if source.kind() == io::ErrorKind::NotFound =>
            {
                return Err(SourceStructureError::SourceFileChanged(self.path.clone()));
            }
            Err(OpenRepositoryFileError::Io(source)) => Err(source),
            Err(OpenRepositoryFileError::NonRegular) => {
                return Err(SourceStructureError::SourceFileChanged(self.path.clone()));
            }
        }
        .map_err(|source| SourceStructureError::Inspect {
            path: self.path.clone(),
            source,
        })?;
        if observed == self.identity && current == self.identity {
            Ok(())
        } else {
            Err(SourceStructureError::SourceFileChanged(self.path.clone()))
        }
    }
}

fn admit_execution(
    path: &Path,
    tracked: Option<TrackedFileMode>,
    worktree: FileExecution,
) -> Result<(), SourceStructureError> {
    match tracked {
        Some(TrackedFileMode::Regular(tracked)) if tracked != worktree => {
            Err(mode_changed(path, tracked.label(), worktree.label()))
        }
        Some(TrackedFileMode::NonRegular) => {
            Err(mode_changed(path, "nonregular", worktree.label()))
        }
        Some(TrackedFileMode::Regular(_)) | None => Ok(()),
    }
}

fn mode_changed(
    path: &Path,
    tracked: &'static str,
    worktree: &'static str,
) -> SourceStructureError {
    SourceStructureError::ExecutionModeChanged {
        path: path.to_owned(),
        tracked,
        worktree,
    }
}

impl FileExecution {
    const fn label(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::NonExecutable => "nonexecutable",
        }
    }
}

fn file_execution(metadata: &Metadata) -> FileExecution {
    if metadata.permissions().mode() & 0o111 == 0 {
        FileExecution::NonExecutable
    } else {
        FileExecution::Executable
    }
}
