use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::string::FromUtf8Error;

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
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    InvalidPath(String),
    NonRegular(PathBuf),
    RunGit {
        operation: &'static str,
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
            Self::RunGit { operation, .. } => {
                write!(formatter, "cannot run `{operation}`")
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
    let relative_path = admitted_relative_path(relative)?;
    let path = repository_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path).map_err(|source| SourceStructureError::Inspect {
        path: path.clone(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(SourceStructureError::NonRegular(path));
    }
    let file = File::open(&path).map_err(|source| SourceStructureError::Inspect {
        path: path.clone(),
        source,
    })?;
    line_count(BufReader::new(file))
        .map_err(|source| SourceStructureError::Inspect { path, source })
}

fn git_paths(
    repository_root: &Path,
    arguments: &[&str],
    operation: &'static str,
) -> Result<BTreeSet<String>, SourceStructureError> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository_root)
        .output()
        .map_err(|source| SourceStructureError::RunGit { operation, source })?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|source| SourceStructureError::GitOutput { operation, source })?;
        return Err(SourceStructureError::GitFailed {
            operation,
            code: output.status.code(),
            stderr,
        });
    }
    String::from_utf8(output.stdout)
        .map_err(|source| SourceStructureError::GitOutput { operation, source })
        .map(|paths| {
            paths
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
                .collect()
        })
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
    let mut lines = 0_u64;
    let mut saw_bytes = false;
    let mut ended_with_newline = false;
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        saw_bytes = true;
        ended_with_newline = buffer.last() == Some(&b'\n');
        for byte in buffer {
            if *byte == b'\n' {
                lines = lines
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source line count overflow"))?;
            }
        }
        let consumed = buffer.len();
        reader.consume(consumed);
    }
    if saw_bytes && !ended_with_newline {
        lines
            .checked_add(1)
            .ok_or_else(|| io::Error::other("source line count overflow"))
    } else {
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::Path;

    use super::{PRESENT_PATH_ARGUMENTS, exceeds_hard_limit, is_source_module, line_count};

    #[test]
    fn line_count_observes_empty_and_final_newline_edges() {
        assert_eq!(line_count(Cursor::new(b"")).ok(), Some(0));
        assert_eq!(line_count(Cursor::new(b"a")).ok(), Some(1));
        assert_eq!(line_count(Cursor::new(b"a\n")).ok(), Some(1));
        assert_eq!(line_count(Cursor::new(b"a\nb")).ok(), Some(2));
        assert_eq!(line_count(Cursor::new(b"a\nb\n")).ok(), Some(2));
    }

    #[test]
    fn source_module_limit_accepts_five_hundred_and_refuses_five_hundred_one() {
        assert!(!exceeds_hard_limit(500));
        assert!(exceeds_hard_limit(501));
    }

    #[test]
    fn source_module_classification_is_explicit() {
        assert!(is_source_module(Path::new("src/lib.rs")));
        assert!(is_source_module(Path::new("scripts/check.py")));
        assert!(is_source_module(Path::new("scripts/check.sh")));
        assert!(!is_source_module(Path::new("README.md")));
        assert!(!is_source_module(Path::new("src/lib.RS")));
    }

    #[test]
    fn source_selection_ignores_only_repository_owned_patterns() {
        assert_eq!(
            PRESENT_PATH_ARGUMENTS,
            [
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-per-directory=.gitignore",
            ]
        );
    }
}
