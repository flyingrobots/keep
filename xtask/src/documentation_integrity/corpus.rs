//! This module owns deterministic documentation source selection.

use std::io;

use xtask::protocol_admission::posix_relative_path;

use super::error::DocumentationError;
use crate::git_inventory::{GitPath, paths_with};
use crate::repository_file::{OpenRepositoryFileError, RepositoryProcessDirectory, RepositoryRoot};

const MARKDOWN_PRESENT: [&str; 7] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
    "--",
    "*.md",
];
const MARKDOWN_DELETED: [&str; 5] = ["ls-files", "-z", "--deleted", "--", "*.md"];
const WORKFLOW_PRESENT: [&str; 8] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
    "--",
    ".github/workflows/*.yml",
    ".github/workflows/*.yaml",
];
const WORKFLOW_DELETED: [&str; 6] = [
    "ls-files",
    "-z",
    "--deleted",
    "--",
    ".github/workflows/*.yml",
    ".github/workflows/*.yaml",
];

pub(super) struct SourceCorpus {
    paths: Vec<String>,
}

#[derive(Clone, Copy)]
enum CorpusKind {
    Markdown,
    Workflow,
}

impl SourceCorpus {
    pub(super) fn markdown(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<Self, DocumentationError> {
        Self::read(repository_root, process_directory, CorpusKind::Markdown)
    }

    pub(super) fn workflow(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<Self, DocumentationError> {
        Self::read(repository_root, process_directory, CorpusKind::Workflow)
    }

    pub(super) fn paths(&self) -> &[String] {
        &self.paths
    }

    fn read(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
        kind: CorpusKind,
    ) -> Result<Self, DocumentationError> {
        let present = paths_with(
            kind.present_arguments(),
            kind.present_operation(),
            |command| process_directory.spawn(command),
        )?;
        let deleted = paths_with(
            kind.deleted_arguments(),
            kind.deleted_operation(),
            |command| process_directory.spawn(command),
        )?;
        let selected = present.difference(&deleted);
        let paths = admit_paths(repository_root, selected, kind)?;
        if paths.is_empty() {
            Err(DocumentationError::EmptyCorpus(kind.label()))
        } else {
            Ok(Self { paths })
        }
    }
}

impl CorpusKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Workflow => "GitHub Actions workflow",
        }
    }

    const fn present_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Markdown => &MARKDOWN_PRESENT,
            Self::Workflow => &WORKFLOW_PRESENT,
        }
    }

    const fn deleted_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Markdown => &MARKDOWN_DELETED,
            Self::Workflow => &WORKFLOW_DELETED,
        }
    }

    const fn present_operation(self) -> &'static str {
        match self {
            Self::Markdown => "git Markdown present paths",
            Self::Workflow => "git workflow present paths",
        }
    }

    const fn deleted_operation(self) -> &'static str {
        match self {
            Self::Markdown => "git Markdown deleted paths",
            Self::Workflow => "git workflow deleted paths",
        }
    }
}

fn admit_paths<'a>(
    repository_root: &RepositoryRoot,
    paths: impl Iterator<Item = &'a GitPath>,
    kind: CorpusKind,
) -> Result<Vec<String>, DocumentationError> {
    paths
        .filter_map(|path| match admit_path(repository_root, path, kind) {
            Ok(Some(path)) => Some(Ok(path)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn admit_path(
    repository_root: &RepositoryRoot,
    path: &GitPath,
    kind: CorpusKind,
) -> Result<Option<String>, DocumentationError> {
    let text = String::from_utf8(path.as_bytes().to_vec()).map_err(|source| {
        DocumentationError::PathEncoding {
            corpus: kind.label(),
            source,
        }
    })?;
    let relative = posix_relative_path(&text).map_err(|_| DocumentationError::InvalidPath {
        corpus: kind.label(),
        path: text.clone(),
    })?;
    match repository_root.open_file(&relative) {
        Ok(_file) => Ok(Some(text)),
        Err(OpenRepositoryFileError::Io(source)) if source.kind() == io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(OpenRepositoryFileError::Io(source))
            if source.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) =>
        {
            Err(DocumentationError::NonRegular {
                corpus: kind.label(),
                path: text,
            })
        }
        Err(OpenRepositoryFileError::Io(source)) => Err(DocumentationError::Inspect {
            corpus: kind.label(),
            path: text,
            source,
        }),
        Err(OpenRepositoryFileError::NonRegular) => Err(DocumentationError::NonRegular {
            corpus: kind.label(),
            path: text,
        }),
    }
}

#[cfg(test)]
#[path = "corpus/tests.rs"]
mod tests;
