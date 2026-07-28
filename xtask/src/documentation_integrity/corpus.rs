//! This module owns deterministic documentation source selection.

use std::fs;
use std::io;
use std::path::Path;

use xtask::protocol_admission::posix_relative_path;

use super::error::DocumentationError;
use crate::git_inventory::{GitPath, paths};

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
    pub(super) fn markdown(repository_root: &Path) -> Result<Self, DocumentationError> {
        Self::read(repository_root, CorpusKind::Markdown)
    }

    pub(super) fn workflow(repository_root: &Path) -> Result<Self, DocumentationError> {
        Self::read(repository_root, CorpusKind::Workflow)
    }

    pub(super) fn paths(&self) -> &[String] {
        &self.paths
    }

    fn read(repository_root: &Path, kind: CorpusKind) -> Result<Self, DocumentationError> {
        let present = paths(
            repository_root,
            kind.present_arguments(),
            kind.present_operation(),
        )?;
        let deleted = paths(
            repository_root,
            kind.deleted_arguments(),
            kind.deleted_operation(),
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
    repository_root: &Path,
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
    repository_root: &Path,
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
    let metadata = match fs::symlink_metadata(repository_root.join(relative)) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(DocumentationError::Inspect {
                corpus: kind.label(),
                path: text,
                source,
            });
        }
    };
    if metadata.file_type().is_file() {
        Ok(Some(text))
    } else {
        Err(DocumentationError::NonRegular {
            corpus: kind.label(),
            path: text,
        })
    }
}

#[cfg(test)]
#[path = "corpus/tests.rs"]
mod tests;
