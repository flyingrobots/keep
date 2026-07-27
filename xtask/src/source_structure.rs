mod git_path_inventory;
mod source_file;

use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::string::FromUtf8Error;

use git_path_inventory::git_paths;
use source_file::{OpenSourceError, open_source_file};

const SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500;
const SOURCE_SUFFIXES: [&str; 3] = ["py", "rs", "sh"];
const PRESENT_PATH_ARGUMENTS: [&str; 5] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
];

pub(super) enum SourceStructureError {
    GitFailed {
        operation: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    GitOutput {
        operation: &'static str,
        source: FromUtf8Error,
    },
    GitOutputBound {
        operation: &'static str,
        stream: &'static str,
        maximum: usize,
    },
    GitOutputFraming {
        operation: &'static str,
    },
    GitPipe {
        operation: &'static str,
        stream: &'static str,
    },
    GitWorker {
        operation: &'static str,
    },
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    InvalidPath(String),
    NonRegular(PathBuf),
    RunGit {
        operation: &'static str,
        action: &'static str,
        source: io::Error,
    },
    Violations(Vec<(String, u64)>),
}

impl fmt::Debug for SourceStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for SourceStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitFailed {
                operation,
                code,
                stderr,
            } => write!(
                formatter,
                "`{operation}` failed with code {code:?}: {}",
                stderr.trim()
            ),
            Self::GitOutput { operation, .. } => {
                write!(formatter, "`{operation}` returned a non-UTF-8 path")
            }
            Self::GitOutputBound {
                operation,
                stream,
                maximum,
            } => write!(
                formatter,
                "`{operation}` exceeded the {maximum}-byte or item {stream} bound"
            ),
            Self::GitOutputFraming { operation } => {
                write!(
                    formatter,
                    "`{operation}` returned a non-NUL-terminated path"
                )
            }
            Self::GitPipe { operation, stream } => {
                write!(formatter, "`{operation}` did not provide its {stream} pipe")
            }
            Self::GitWorker { operation } => {
                write!(
                    formatter,
                    "`{operation}` diagnostic reader stopped unexpectedly"
                )
            }
            Self::Inspect { path, .. } => {
                write!(formatter, "cannot inspect `{}`", path.display())
            }
            Self::InvalidPath(path) => {
                write!(formatter, "git returned unsafe path `{path}`")
            }
            Self::NonRegular(path) => write!(
                formatter,
                "tracked source module is not a regular file: `{}`",
                path.display()
            ),
            Self::RunGit {
                operation, action, ..
            } => {
                write!(formatter, "cannot {action} `{operation}`")
            }
            Self::Violations(violations) => {
                formatter.write_str("tracked source modules exceed the 500-line hard maximum")?;
                for (path, lines) in violations {
                    write!(formatter, "; {path}: {lines}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for SourceStructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GitOutput { source, .. } => Some(source),
            Self::Inspect { source, .. } | Self::RunGit { source, .. } => Some(source),
            Self::GitFailed { .. }
            | Self::GitOutputBound { .. }
            | Self::GitOutputFraming { .. }
            | Self::GitPipe { .. }
            | Self::GitWorker { .. }
            | Self::InvalidPath(_)
            | Self::NonRegular(_)
            | Self::Violations(_) => None,
        }
    }
}

pub(super) fn check(repository_root: &Path) -> Result<(), SourceStructureError> {
    let paths = source_paths(repository_root)?;
    let violations = source_violations(repository_root, paths)?;
    if violations.is_empty() {
        Ok(())
    } else {
        Err(SourceStructureError::Violations(violations))
    }
}

fn source_paths(repository_root: &Path) -> Result<Vec<String>, SourceStructureError> {
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
    Ok(present
        .difference(&deleted)
        .filter(|path| is_source_module(Path::new(path)))
        .cloned()
        .collect())
}

fn source_violations(
    repository_root: &Path,
    paths: Vec<String>,
) -> Result<Vec<(String, u64)>, SourceStructureError> {
    let mut violations = Vec::new();
    for relative in paths {
        let lines = source_line_count(repository_root, &relative)?;
        if exceeds_hard_limit(lines) {
            violations.push((relative, lines));
        }
    }
    Ok(violations)
}

fn source_line_count(repository_root: &Path, relative: &str) -> Result<u64, SourceStructureError> {
    source_line_count_with(repository_root, relative, open_source_file)
}

fn source_line_count_with(
    repository_root: &Path,
    relative: &str,
    open_source: impl FnOnce(&Path, &Path) -> Result<std::fs::File, OpenSourceError>,
) -> Result<u64, SourceStructureError> {
    let relative_path = admitted_relative_path(relative)?;
    let path = repository_root.join(relative_path);
    let file = open_source(repository_root, relative_path).map_err(|error| match error {
        OpenSourceError::Io(source) => SourceStructureError::Inspect {
            path: path.clone(),
            source,
        },
        OpenSourceError::NonRegular => SourceStructureError::NonRegular(path.clone()),
    })?;
    line_count(BufReader::new(file))
        .map_err(|source| SourceStructureError::Inspect { path, source })
}

fn admitted_relative_path(path: &str) -> Result<&Path, SourceStructureError> {
    let relative = Path::new(path);
    if relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(relative)
    } else {
        Err(SourceStructureError::InvalidPath(path.to_owned()))
    }
}

fn is_source_module(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|suffix| SOURCE_SUFFIXES.contains(&suffix))
}

const fn exceeds_hard_limit(lines: u64) -> bool {
    lines > SOURCE_MODULE_HARD_LIMIT_LINES
}

fn line_count(mut reader: impl BufRead) -> Result<u64, io::Error> {
    let mut completed_lines = 0_u64;
    let mut in_line = false;
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        for byte in buffer {
            if !in_line {
                let current_line = completed_lines
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source line count overflow"))?;
                if exceeds_hard_limit(current_line) {
                    return Ok(current_line);
                }
                in_line = true;
            }
            if *byte == b'\n' {
                completed_lines = completed_lines
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source line count overflow"))?;
                in_line = false;
            }
        }
        let consumed = buffer.len();
        reader.consume(consumed);
    }
    if in_line {
        completed_lines
            .checked_add(1)
            .ok_or_else(|| io::Error::other("source line count overflow"))
    } else {
        Ok(completed_lines)
    }
}

#[cfg(test)]
mod tests;
